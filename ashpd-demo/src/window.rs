use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};

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
        DocumentsPage,
    },
    widgets::SidebarRow,
};

mod imp {

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub documents: TemplateChild<DocumentsPage>,
        #[template_child]
        pub(super) dynamic_launcher: TemplateChild<DynamicLauncherPage>,
        #[template_child]
        pub screenshot: TemplateChild<ScreenshotPage>,
        #[template_child]
        pub camera: TemplateChild<CameraPage>,
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
        pub account: TemplateChild<AccountPage>,
        #[template_child]
        pub email: TemplateChild<EmailPage>,
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
        #[template_child]
        pub color_scheme_btn: TemplateChild<gtk::Button>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "ApplicationWindow";
        type Type = super::ApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                screenshot: TemplateChild::default(),
                stack: TemplateChild::default(),
                window_title: TemplateChild::default(),
                split_view: TemplateChild::default(),
                camera: TemplateChild::default(),
                dynamic_launcher: TemplateChild::default(),
                usb: TemplateChild::default(),
                wallpaper: TemplateChild::default(),
                location: TemplateChild::default(),
                notification: TemplateChild::default(),
                screencast: TemplateChild::default(),
                documents: TemplateChild::default(),
                account: TemplateChild::default(),
                email: TemplateChild::default(),
                file_chooser: TemplateChild::default(),
                open_uri: TemplateChild::default(),
                inhibit: TemplateChild::default(),
                global_shortcuts: TemplateChild::default(),
                secret: TemplateChild::default(),
                remote_desktop: TemplateChild::default(),
                print: TemplateChild::default(),
                color_scheme_btn: TemplateChild::default(),
                settings: gio::Settings::new(config::APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            SidebarRow::static_type();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            if config::PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
            let is_sandboxed: bool =
                glib::MainContext::default().block_on(async { ashpd::is_sandboxed().await });
            // Add pages based on whether the app is sandboxed
            if is_sandboxed {
                self.stack.add_titled(
                    &BackgroundPage::default(),
                    Some("background"),
                    &gettext("Background"),
                );
            } else {
                self.stack
                    .add_titled(&DevicePage::default(), Some("device"), &gettext("Device"));

                self.stack.add_titled(
                    &NetworkMonitorPage::default(),
                    Some("network_monitor"),
                    &gettext("Network Monitor"),
                );

                self.stack.add_titled(
                    &ProxyResolverPage::default(),
                    Some("proxy_resolver"),
                    &gettext("Proxy Resolver"),
                );
            }

            self.stack.set_visible_child_name("welcome");
            // load latest window state
            let button = self.color_scheme_btn.get();
            let style_manager = adw::StyleManager::default();

            if !style_manager.system_supports_color_schemes() {
                button.set_visible(true);

                style_manager.connect_dark_notify(clone!(
                    #[weak]
                    button,
                    move |manager| {
                        if manager.is_dark() {
                            button.set_icon_name("light-mode-symbolic");
                        } else {
                            button.set_icon_name("dark-mode-symbolic");
                        }
                    }
                ));
            }
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
        let settings = &self.imp().settings;

        let size = self.default_size();

        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn sidebar_row_selected(&self, row: &gtk::ListBoxRow) {
        let imp = self.imp();

        let sidebar_row = row.downcast_ref::<SidebarRow>().unwrap();
        imp.split_view.set_show_content(true);
        let page_name = sidebar_row.page_name();
        if imp.stack.child_by_name(&page_name).is_some() {
            imp.stack.set_visible_child_name(&page_name);
            imp.window_title.set_title(&sidebar_row.title());
        } else {
            imp.window_title.set_title("");
            imp.stack.set_visible_child_name("welcome");
        }
    }

    fn load_window_state(&self) {
        let settings = &self.imp().settings;

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
        let style_manager = adw::StyleManager::default();
        if !style_manager.system_supports_color_schemes() {
            if settings.boolean("dark-mode") {
                style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
            } else {
                style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
            }
        }
    }
}
