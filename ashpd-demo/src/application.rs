use std::{collections::HashMap, convert::TryFrom};

use ashpd::{
    flatpak::{FlatpakProxy, SpawnFlags, SpawnOptions},
    zbus, zvariant,
};
use gio::ApplicationFlags;
use glib::clone;
use glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::{action, stateful_action};
use once_cell::sync::OnceCell;
use serde::Serialize;
use tracing::{debug, info};

use crate::config;
use crate::window::ApplicationWindow;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

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
        fn constructed(&self, obj: &Self::Type) {
            obj.add_main_option(
                "replace",
                glib::Char::try_from('r').unwrap(),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Replace the running instance",
                None,
            );
            self.parent_constructed(obj);
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self, app: &Self::Type) {
            debug!("Application::activate");

            let priv_ = Application::from_instance(app);
            if let Some(window) = priv_.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }
            let window = ApplicationWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();

            app.main_window().present();
            self.parent_activate(app);
        }

        fn startup(&self, app: &Self::Type) {
            debug!("Application::startup");
            adw::init();
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/com/belmoussaoui/ashpd/demo/style.css");
            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);

            if let Some(ref display) = gtk::gdk::Display::default() {
                gtk::StyleContext::add_provider_for_display(
                    display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }

            self.settings
                .connect_changed(Some("dark-mode"), |_settings, _key| {
                    let style_manager = adw::StyleManager::default().unwrap();
                    if style_manager.is_dark() {
                        style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                    } else {
                        style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
                    }
                });
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::FLAGS_NONE),
            ("resource-base-path", &Some("/com/belmoussaoui/ashpd/demo/")),
        ])
        .expect("Application initialization failed...")
    }

    fn main_window(&self) -> ApplicationWindow {
        let priv_ = imp::Application::from_instance(self);
        priv_.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        let self_ = imp::Application::from_instance(self);
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
        // The restart app requires the Flatpak portal
        gtk_macros::get_action!(self, @restart).set_enabled(ashpd::is_sandboxed());

        let is_dark_mode = self_.settings.boolean("dark-mode");
        stateful_action!(
            self,
            "dark-mode",
            is_dark_mode,
            clone!(@weak self_.settings as settings =>  move |action, _| {
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_dark_mode = !action_state;
                action.set_state(&is_dark_mode.to_variant());
                if let Err(err) = settings.set_boolean("dark-mode", is_dark_mode) {
                    tracing::error!("Failed to switch dark mode: {} ", err);
                }
            })
        );

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
        let bus = gio::bus_get_sync(gio::BusType::Session, gio::NONE_CANCELLABLE).unwrap();
        gio::bus_watch_name_on_connection(
            &bus,
            config::APP_ID,
            gio::BusNameWatcherFlags::NONE,
            move |_bus, _, _| tracing::info!("Name owned"),
            move |_bus, _| tracing::info!("Name unowned"),
        );

        let cnx = zbus::Connection::session()?;
        let proxy: zbus::Proxy = zbus::ProxyBuilder::new_bare(&cnx)
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
        let dialog = gtk::AboutDialogBuilder::new()
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::MitX11)
            .website("https://github.com/bilelmoussaoui/ashpd/")
            .version(config::VERSION)
            .transient_for(&self.main_window())
            .modal(true)
            .authors(vec!["Bilal Elmoussaoui".into()])
            .build();

        dialog.show();
    }

    async fn restart(&self) -> ashpd::Result<()> {
        let cnx = zbus::Connection::session().await?;
        let proxy = FlatpakProxy::new(&cnx).await?;
        proxy
            .spawn(
                "/",
                &["ashpd-demo", "--replace"],
                HashMap::new(),
                HashMap::new(),
                SpawnFlags::Latest.into(),
                SpawnOptions::default(),
            )
            .await?;
        Ok(())
    }

    pub fn run(&self) {
        info!("ASHPD Demo ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
