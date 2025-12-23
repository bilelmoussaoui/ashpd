use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio, glib};

use crate::{
    application::Application,
    config,
    portals::{
        desktop::{
            AccountPage, BackgroundPage, CameraPage, DevicePage, DynamicLauncherPage, EmailPage,
            FileChooserPage, GlobalShortcutsPage, InhibitPage, LocationPage, NetworkMonitorPage,
            NotificationPage, OpenUriPage, PrintPage, ProxyResolverPage, RemoteDesktopPage,
            ScreenCastPage, ScreenshotPage, SecretPage, UsbPage, WallpaperPage,
        },
        spawn_tokio_blocking, DocumentsPage,
    },
};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub account: TemplateChild<AccountPage>,
        #[template_child]
        pub background_page: TemplateChild<adw::ViewStackPage>,
        #[template_child]
        pub camera: TemplateChild<CameraPage>,
        #[template_child]
        pub device_page: TemplateChild<adw::ViewStackPage>,
        #[template_child]
        pub documents: TemplateChild<DocumentsPage>,
        #[template_child]
        pub(super) dynamic_launcher: TemplateChild<DynamicLauncherPage>,
        #[template_child]
        pub email: TemplateChild<EmailPage>,
        #[template_child]
        pub network_monitor_page: TemplateChild<adw::ViewStackPage>,
        #[template_child]
        pub proxy_resolver_page: TemplateChild<adw::ViewStackPage>,
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub screenshot: TemplateChild<ScreenshotPage>,
        #[template_child]
        pub usb: TemplateChild<UsbPage>,
        #[template_child]
        pub wallpaper: TemplateChild<WallpaperPage>,
        #[template_child]
        pub location: TemplateChild<LocationPage>,
        #[template_child]
        pub notification: TemplateChild<NotificationPage>,
        #[template_child]
        pub screencast: TemplateChild<ScreenCastPage>,
        #[template_child]
        pub file_chooser: TemplateChild<FileChooserPage>,
        #[template_child]
        pub open_uri: TemplateChild<OpenUriPage>,
        #[template_child]
        pub inhibit: TemplateChild<InhibitPage>,
        #[template_child]
        pub global_shortcuts: TemplateChild<GlobalShortcutsPage>,
        #[template_child]
        pub secret: TemplateChild<SecretPage>,
        #[template_child]
        pub remote_desktop: TemplateChild<RemoteDesktopPage>,
        #[template_child]
        pub print: TemplateChild<PrintPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "ApplicationWindow";
        type Type = super::ApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            BackgroundPage::static_type();
            DevicePage::static_type();
            NetworkMonitorPage::static_type();
            ProxyResolverPage::static_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl ApplicationWindow {
        #[template_callback]
        fn view_switcher_sidebar_activated(&self) {
            self.split_view.set_show_content(true);
        }
    }

    impl ObjectImpl for ApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            if config::PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
            let is_sandboxed: bool = spawn_tokio_blocking(async { ashpd::is_sandboxed().await });
            // Add pages based on whether the app is sandboxed
            self.background_page.set_visible(is_sandboxed);
            self.device_page.set_visible(!is_sandboxed);
            self.network_monitor_page.set_visible(!is_sandboxed);
            self.proxy_resolver_page.set_visible(!is_sandboxed);

            self.stack.set_visible_child_name("welcome");
            obj.load_window_state();
        }
    }

    impl WidgetImpl for ApplicationWindow {}
    impl WindowImpl for ApplicationWindow {
        // save window state on delete event
        fn close_request(&self) -> glib::Propagation {
            if let Err(err) = self.obj().save_window_size() {
                tracing::warn!("Failed to save window state, {}", &err);
            }
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for ApplicationWindow {}
    impl AdwApplicationWindowImpl for ApplicationWindow {}
}

glib::wrapper! {
    pub struct ApplicationWindow(ObjectSubclass<imp::ApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable, gtk::ShortcutManager, gtk::Native, gtk::Root;
}

impl ApplicationWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let source = gio::SettingsSchemaSource::default().unwrap();
        if source.lookup(config::APP_ID, false).is_some() {
            let settings = gio::Settings::new(config::APP_ID);

            let size = self.default_size();

            settings.set_int("window-width", size.0)?;
            settings.set_int("window-height", size.1)?;

            settings.set_boolean("is-maximized", self.is_maximized())?;
        }

        Ok(())
    }

    fn load_window_state(&self) {
        let source = gio::SettingsSchemaSource::default().unwrap();
        if source.lookup(config::APP_ID, false).is_some() {
            let settings = gio::Settings::new(config::APP_ID);

            let width = settings.int("window-width");
            let height = settings.int("window-height");
            let is_maximized = settings.boolean("is-maximized");

            self.set_default_size(width, height);

            if is_maximized {
                self.maximize();
            }
        }
    }
}
