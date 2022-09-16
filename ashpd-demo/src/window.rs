use gettextrs::gettext;
use glib::signal::Inhibit;
use gtk::{
    self, gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use tracing::warn;

use crate::{
    application::Application,
    config,
    portals::{
        desktop::{
            AccountPage, BackgroundPage, CameraPage, DevicePage, EmailPage, FileChooserPage,
            InhibitPage, LocationPage, NetworkMonitorPage, NotificationPage, OpenUriPage,
            PrintPage, ProxyResolverPage, RemoteDesktopPage, ScreenCastPage, ScreenshotPage,
            SecretPage, WallpaperPage,
        },
        DocumentsPage,
    },
    widgets::SidebarRow,
};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub sidebar: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub documents: TemplateChild<DocumentsPage>,
        #[template_child]
        pub screenshot: TemplateChild<ScreenshotPage>,
        #[template_child]
        pub camera: TemplateChild<CameraPage>,
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
                sidebar: TemplateChild::default(),
                window_title: TemplateChild::default(),
                leaflet: TemplateChild::default(),
                camera: TemplateChild::default(),
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

            klass.install_action("win.back", None, |win, _, _| {
                win.imp().leaflet.navigate(adw::NavigationDirection::Back);
            });
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            if config::PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
            let is_sandboxed = futures::executor::block_on(async { ashpd::is_sandboxed().await });
            // Add pages based on whether the app is sandboxed
            if is_sandboxed {
                self.sidebar
                    .insert(&SidebarRow::new(&gettext("Background"), "background"), 1);
                self.stack
                    .add_named(&BackgroundPage::new(), Some("background"));
            } else {
                self.sidebar
                    .insert(&SidebarRow::new(&gettext("Device"), "device"), 2);
                self.stack.add_named(&DevicePage::new(), Some("device"));

                self.sidebar.insert(
                    &SidebarRow::new(&gettext("Network Monitor"), "network_monitor"),
                    7,
                );
                self.stack
                    .add_named(&NetworkMonitorPage::new(), Some("network_monitor"));

                self.sidebar.insert(
                    &SidebarRow::new(&gettext("Proxy Resolver"), "proxy_resolver"),
                    11,
                );

                self.stack
                    .add_named(&ProxyResolverPage::new(), Some("proxy_resolver"));
            }

            let row = self.sidebar.row_at_index(0).unwrap();
            self.sidebar.unselect_row(&row);
            self.sidebar
                .connect_row_activated(clone!(@weak obj as win => move |_, row| {
                    win.sidebar_row_selected(row);
                }));
            self.stack.set_visible_child_name("welcome");
            // load latest window state
            let button = self.color_scheme_btn.get();
            let style_manager = adw::StyleManager::default();

            if !style_manager.system_supports_color_schemes() {
                button.show();

                style_manager.connect_dark_notify(
                    clone!(@weak obj as window, @weak button => move |manager| {
                        if manager.is_dark() {
                            button.set_icon_name("light-mode-symbolic");
                        } else {
                            button.set_icon_name("dark-mode-symbolic");
                        }
                    }),
                );
            }
            obj.load_window_state();
        }
    }

    impl WidgetImpl for ApplicationWindow {}
    impl WindowImpl for ApplicationWindow {
        // save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            if let Err(err) = obj.save_window_size() {
                warn!("Failed to save window state, {}", &err);
            }
            Inhibit(false)
        }
    }

    impl ApplicationWindowImpl for ApplicationWindow {}
    impl AdwApplicationWindowImpl for ApplicationWindow {}
}

glib::wrapper! {
    pub struct ApplicationWindow(ObjectSubclass<imp::ApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl ApplicationWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", &app)]).expect("Failed to create ApplicationWindow")
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
        imp.leaflet.navigate(adw::NavigationDirection::Forward);
        let page_name = sidebar_row.name();
        if imp.stack.child_by_name(&page_name).is_some() {
            imp.stack.set_visible_child_name(&page_name);
            imp.window_title
                .set_title(&sidebar_row.title().unwrap_or_default());
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
