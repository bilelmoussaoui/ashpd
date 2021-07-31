use std::collections::HashMap;

use ashpd::{
    flatpak::{FlatpakProxy, SpawnFlags, SpawnOptions},
    zbus,
};
use gio::ApplicationFlags;
use glib::clone;
use glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::{action, stateful_action};
use once_cell::sync::OnceCell;
use tracing::{debug, info};

use crate::config;
use crate::window::ExampleApplicationWindow;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct ExampleApplication {
        pub window: OnceCell<WeakRef<ExampleApplicationWindow>>,
        pub settings: gio::Settings,
    }

    impl Default for ExampleApplication {
        fn default() -> Self {
            Self {
                window: OnceCell::default(),
                settings: gio::Settings::new(config::APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExampleApplication {
        const NAME: &'static str = "ExampleApplication";
        type Type = super::ExampleApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for ExampleApplication {}

    impl ApplicationImpl for ExampleApplication {
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<ExampleApplication>::activate");

            let priv_ = ExampleApplication::from_instance(app);
            if let Some(window) = priv_.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }
            let window = ExampleApplicationWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();

            app.main_window().present();
            self.parent_activate(app);
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<ExampleApplication>::startup");
            adw::init();
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/com/belmoussaoui/ashpd/demo/style.css");

            if let Some(ref display) = gtk::gdk::Display::default() {
                gtk::StyleContext::add_provider_for_display(
                    display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }

            let settings = gtk::Settings::default().unwrap();
            self.settings
                .bind("dark-mode", &settings, "gtk-application-prefer-dark-theme")
                .build();
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for ExampleApplication {}
}

glib::wrapper! {
    pub struct ExampleApplication(ObjectSubclass<imp::ExampleApplication>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl ExampleApplication {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::FLAGS_NONE),
            ("resource-base-path", &Some("/com/belmoussaoui/ashpd/demo/")),
        ])
        .expect("Application initialization failed...")
    }

    fn main_window(&self) -> ExampleApplicationWindow {
        let priv_ = imp::ExampleApplication::from_instance(self);
        priv_.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        let self_ = imp::ExampleApplication::from_instance(self);
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
                    app.restart().await;
                }));
            })
        );

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

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.dark-mode", &["<primary>T"]);
        self.set_accels_for_action("app.quit", &["<primary>q"]);
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
            .artists(vec!["Bilal Elmoussaoui".into()])
            .build();

        dialog.show();
    }

    async fn restart(&self) -> ashpd::Result<()> {
        let cnx = zbus::azync::Connection::session().await?;
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
