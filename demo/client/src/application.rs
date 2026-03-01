use std::{collections::HashMap, convert::TryFrom, str::FromStr, sync::Arc};

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    flatpak::{
        Flatpak, SpawnFlags, SpawnOptions,
        update_monitor::{UpdateInfo, UpdateMonitor},
    },
    zbus, zvariant,
};
use futures_util::StreamExt;
use gtk::{
    gio,
    glib::{self, clone},
};

use crate::{config, portals::spawn_tokio, update_window::UpdateWindow, window::ApplicationWindow};

mod imp {
    use std::cell::OnceCell;

    use futures_util::lock::Mutex;
    use glib::WeakRef;

    use super::*;

    #[derive(Debug, Default)]
    pub struct Application {
        pub window: OnceCell<WeakRef<ApplicationWindow>>,
        pub update_monitor: Arc<Mutex<Option<Arc<UpdateMonitor>>>>,
        pub update_info: Arc<Mutex<Option<UpdateInfo>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_main_option(
                "replace",
                glib::Char::try_from('r').unwrap(),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Replace the running instance",
                None,
            );
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();
            let app = self.obj();
            tracing::debug!("Application::activate");

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }
            let window = ApplicationWindow::new(&app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();
            app.setup_update_monitor();

            app.main_window().present();
        }

        fn startup(&self) {
            self.parent_startup();
            tracing::debug!("Application::startup");
            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    fn main_window(&self) -> ApplicationWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event
                // and saving the window state
                app.main_window().close();
                app.quit();
            })
            .build();

        // About
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about_dialog())
            .build();

        let restart_action = gio::ActionEntry::builder("restart")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event
                // and saving the window state
                glib::spawn_future_local(clone!(
                    #[weak]
                    app,
                    async move {
                        if let Err(err) = app.restart().await {
                            tracing::error!("Failed to restart the application {}", err);
                        }
                    }
                ));
            })
            .build();

        let install_update_action = gio::ActionEntry::builder("install-update")
            .activate(move |app: &Self, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    app,
                    async move {
                        app.show_update_dialog().await;
                    }
                ));
            })
            .build();

        self.add_action_entries([
            about_action,
            quit_action,
            restart_action,
            install_update_action,
        ]);

        // The restart app requires the Flatpak portal
        self.lookup_action("restart")
            .and_downcast_ref::<gio::SimpleAction>()
            .unwrap()
            .set_enabled(ashpd::is_sandboxed());

        // Install update requires Flatpak and is disabled by default until an update is
        // available
        self.lookup_action("install-update")
            .and_downcast_ref::<gio::SimpleAction>()
            .unwrap()
            .set_enabled(ashpd::is_sandboxed());
    }

    pub async fn stop_current_instance() -> ashpd::Result<()> {
        spawn_tokio(async move {
            let cnx = zbus::Connection::session().await?;
            let proxy: zbus::Proxy = zbus::proxy::Builder::new(&cnx)
                .path(format!(
                    "/{}",
                    config::APP_ID.split('.').collect::<Vec<_>>().join("/")
                ))?
                .interface("org.gtk.Actions")?
                .destination(config::APP_ID)?
                .build()
                .await?;
            #[derive(Debug, serde::Serialize)]
            pub struct Params(
                String,
                Vec<zvariant::OwnedValue>,
                HashMap<String, zvariant::OwnedValue>,
            );

            impl zvariant::DynamicType for Params {
                fn signature(&self) -> zvariant::Signature {
                    zvariant::Signature::from_str("(sava{sv})").unwrap()
                }
            }

            proxy
                .call_method(
                    "Activate",
                    &Params("quit".to_string(), Vec::new(), HashMap::new()),
                )
                .await
        })
        .await?;
        Ok(())
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.dark-mode", &["<control>T"]);
        self.set_accels_for_action("app.restart", &["<control>R"]);
        self.set_accels_for_action("app.quit", &["<control>Q"]);
        self.set_accels_for_action("window.close", &["<control>W"]);
    }

    fn show_about_dialog(&self) {
        adw::AboutDialog::builder()
            .application_icon(config::APP_ID)
            .license_type(gtk::License::MitX11)
            .website("https://github.com/bilelmoussaoui/ashpd/")
            .version(config::VERSION)
            .developer_name("Bilal Elmoussaoui")
            .build()
            .present(Some(&self.main_window()));
    }

    async fn restart(&self) -> ashpd::Result<()> {
        spawn_tokio(async move {
            let proxy = Flatpak::new().await?;
            let fds: HashMap<u32, std::fs::File> = HashMap::new();
            proxy
                .spawn(
                    "/",
                    &["ashpd-demo", "--replace"],
                    fds,
                    HashMap::new(),
                    SpawnFlags::LatestVersion.into(),
                    SpawnOptions::default(),
                )
                .await
        })
        .await?;
        Ok(())
    }

    fn setup_update_monitor(&self) {
        if !ashpd::is_sandboxed() {
            tracing::debug!("Not sandboxed, skipping update monitor");
            return;
        }

        let imp = self.imp();
        let update_info = imp.update_info.clone();
        let update_monitor = imp.update_monitor.clone();

        let (sender, mut receiver) = tokio::sync::mpsc::channel::<UpdateInfo>(10);

        let app = self.clone();
        glib::spawn_future_local(async move {
            while let Some(info) = receiver.recv().await {
                tracing::info!(
                    "Update available: running={}, remote={}",
                    info.running_commit(),
                    info.remote_commit()
                );

                *update_info.lock().await = Some(info);

                app.lookup_action("install-update")
                    .and_downcast_ref::<gio::SimpleAction>()
                    .unwrap()
                    .set_enabled(true);
            }
        });

        crate::portals::RUNTIME.spawn(async move {
            let monitor = match Flatpak::new().await {
                Ok(proxy) => match proxy.create_update_monitor(Default::default()).await {
                    Ok(m) => m,
                    Err(err) => {
                        tracing::error!("Failed to create update monitor: {err}");
                        return;
                    }
                },
                Err(err) => {
                    tracing::error!("Failed to create Flatpak proxy: {err}");
                    return;
                }
            };

            let monitor = Arc::new(monitor);
            *update_monitor.lock().await = Some(monitor.clone());

            let mut stream = match monitor.receive_update_available().await {
                Ok(stream) => stream,
                Err(err) => {
                    tracing::error!("Failed to receive update_available stream: {err}");
                    return;
                }
            };

            while let Some(info) = stream.next().await {
                if sender.send(info).await.is_err() {
                    break;
                }
            }
        });
    }

    async fn show_update_dialog(&self) {
        let imp = self.imp();

        let (running_commit, remote_commit) = {
            let update_info = imp.update_info.lock().await;
            if let Some(ref info) = *update_info {
                (
                    Some(info.running_commit().to_string()),
                    Some(info.remote_commit().to_string()),
                )
            } else {
                (None, None)
            }
        };

        let monitor = {
            let monitor_guard = imp.update_monitor.lock().await;
            monitor_guard.clone()
        };

        let window = UpdateWindow::new(
            &self.main_window(),
            monitor,
            running_commit.as_deref(),
            remote_commit.as_deref(),
        );
        window.present();
    }

    pub fn run() -> glib::ExitCode {
        tracing::info!("ASHPD Demo ({})", config::APP_ID);
        tracing::info!("Version: {} ({})", config::VERSION, config::PROFILE);

        Self::default().run()
    }
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/com/belmoussaoui/ashpd/demo/")
            .build()
    }
}
