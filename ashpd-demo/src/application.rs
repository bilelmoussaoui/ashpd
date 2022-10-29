use std::{collections::HashMap, convert::TryFrom};

use adw::prelude::*;
use ashpd::{
    flatpak::{Flatpak, SpawnFlags, SpawnOptions},
    zbus, zvariant,
};
use gio::ApplicationFlags;
use glib::{clone, WeakRef};
use gtk::{gio, glib, subclass::prelude::*};
use gtk_macros::action;
use once_cell::sync::OnceCell;
use serde::Serialize;
use tracing::{debug, info};

use crate::{config, window::ApplicationWindow};

mod imp {
    use adw::subclass::prelude::*;

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
            debug!("Application::activate");

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }
            let window = ApplicationWindow::new(&*app);
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
            debug!("Application::startup");
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::FLAGS_NONE),
            ("resource-base-path", &Some("/com/belmoussaoui/ashpd/demo/")),
        ])
    }

    fn main_window(&self) -> ApplicationWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        action!(
            self,
            "quit",
            clone!(@weak self as app => move |_, _| {
                // This is needed to trigger the delete event
                // and saving the window state
                app.main_window().close();
                app.quit();
            })
        );

        action!(
            self,
            "restart",
            clone!(@weak self as app => move |_, _| {
                // This is needed to trigger the delete event
                // and saving the window state
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak app => async move {
                    if let Err(err) = app.restart().await {
                        tracing::error!("Failed to restart the application {}", err);
                    }
                }));
            })
        );
        let is_sandboxed = futures::executor::block_on(async { ashpd::is_sandboxed().await });
        // The restart app requires the Flatpak portal
        gtk_macros::get_action!(self, @restart).set_enabled(is_sandboxed);

        let action = self.imp().settings.create_action("dark-mode");
        self.add_action(&action);
        // About
        action!(
            self,
            "about",
            clone!(@weak self as app => move |_, _| {
                app.show_about_dialog();
            })
        );
    }

    pub fn stop_current_instance() -> ashpd::Result<()> {
        let bus = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).unwrap();
        gio::bus_watch_name_on_connection(
            &bus,
            config::APP_ID,
            gio::BusNameWatcherFlags::NONE,
            move |_bus, _, _| tracing::info!("Name owned"),
            move |_bus, _| tracing::info!("Name unowned"),
        );

        let cnx = zbus::blocking::Connection::session()?;
        let proxy: zbus::blocking::Proxy = zbus::blocking::ProxyBuilder::new_bare(&cnx)
            .path(format!(
                "/{}",
                config::APP_ID.split('.').collect::<Vec<_>>().join("/")
            ))?
            .interface("org.gtk.Actions")?
            .destination(config::APP_ID)?
            .build()?;
        #[derive(Debug, Serialize)]
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

        proxy.call_method(
            "Activate",
            &Params("quit".to_string(), Vec::new(), HashMap::new()),
        )?;

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

    pub fn run(&self) {
        info!("ASHPD Demo ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
