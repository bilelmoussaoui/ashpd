use std::{collections::HashMap, convert::TryFrom};

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    flatpak::{Flatpak, SpawnFlags, SpawnOptions},
    zbus, zvariant,
};
use gtk::{
    gio,
    glib::{self, clone},
};

use crate::{config, window::ApplicationWindow};

mod imp {
    use std::cell::OnceCell;

    use glib::WeakRef;

    use super::*;

    #[derive(Debug)]
    pub struct Application {
        pub window: OnceCell<WeakRef<ApplicationWindow>>,
        pub settings: gio::Settings,
    }

    impl Default for Application {
        fn default() -> Self {
            Self {
                window: OnceCell::default(),
                settings: gio::Settings::new(config::APP_ID),
            }
        }
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

            app.main_window().present();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            tracing::debug!("Application::startup");
            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);
            self.settings.connect_changed(
                Some("dark-mode"),
                clone!(@weak app => move |_, _| {
                    app.update_color_scheme();
                }),
            );
            app.update_color_scheme();
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
                glib::spawn_future_local(clone!(@weak app => async move {
                    if let Err(err) = app.restart().await {
                        tracing::error!("Failed to restart the application {}", err);
                    }
                }));
            })
            .build();

        self.add_action_entries([about_action, quit_action, restart_action]);

        let is_sandboxed =
            glib::MainContext::default().block_on(async { ashpd::is_sandboxed().await });
        // The restart app requires the Flatpak portal
        self.lookup_action("restart")
            .and_downcast_ref::<gio::SimpleAction>()
            .unwrap()
            .set_enabled(is_sandboxed);

        let action = self.imp().settings.create_action("dark-mode");
        self.add_action(&action);
    }

    pub async fn stop_current_instance() -> ashpd::Result<()> {
        let cnx = zbus::Connection::session().await?;
        let proxy: zbus::Proxy = zbus::ProxyBuilder::new(&cnx)
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

        impl zvariant::Type for Params {
            fn signature() -> zvariant::Signature<'static> {
                zvariant::Signature::from_str_unchecked("(sava{sv})")
            }
        }

        proxy
            .call_method(
                "Activate",
                &Params("quit".to_string(), Vec::new(), HashMap::new()),
            )
            .await?;

        Ok(())
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.dark-mode", &["<primary>T"]);
        self.set_accels_for_action("app.restart", &["<primary>R"]);
        self.set_accels_for_action("app.quit", &["<primary>Q"]);
        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
    }

    fn show_about_dialog(&self) {
        adw::AboutWindow::builder()
            .application_icon(config::APP_ID)
            .license_type(gtk::License::MitX11)
            .website("https://github.com/bilelmoussaoui/ashpd/")
            .version(config::VERSION)
            .transient_for(&self.main_window())
            .modal(true)
            .developer_name("Bilal Elmoussaoui")
            .build()
            .present();
    }

    async fn restart(&self) -> ashpd::Result<()> {
        let proxy = Flatpak::new().await?;
        proxy
            .spawn(
                "/",
                &["ashpd-demo", "--replace"],
                HashMap::new(),
                HashMap::new(),
                SpawnFlags::LatestVersion.into(),
                SpawnOptions::default(),
            )
            .await?;
        Ok(())
    }

    fn update_color_scheme(&self) {
        let manager = self.style_manager();
        if !manager.system_supports_color_schemes() {
            let color_scheme = if self.imp().settings.boolean("dark-mode") {
                adw::ColorScheme::PreferDark
            } else {
                adw::ColorScheme::PreferLight
            };
            manager.set_color_scheme(color_scheme);
        }
    }

    pub fn run() -> glib::ExitCode {
        tracing::info!("ASHPD Demo ({})", config::APP_ID);
        tracing::info!("Version: {} ({})", config::VERSION, config::PROFILE);
        tracing::info!("Datadir: {}", config::PKGDATADIR);

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
