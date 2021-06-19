use glib::signal::Inhibit;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use log::warn;

use crate::application::ExampleApplication;
use crate::config::{APP_ID, PROFILE};
use crate::portals::desktop::{
    AccountPage, BackgroundPage, CameraPage, DevicePage, EmailPage, LocationPage,
    NetworkMonitorPage, NotificationPage, OpenUriPage, ScreenCastPage, ScreenshotPage,
    WallpaperPage,
};
use crate::portals::DocumentsPage;
use crate::sidebar_row::SidebarRow;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib::clone;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/window.ui")]
    pub struct ExampleApplicationWindow {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub sidebar: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub screenshot: TemplateChild<ScreenshotPage>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExampleApplicationWindow {
        const NAME: &'static str = "ExampleApplicationWindow";
        type Type = super::ExampleApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                screenshot: TemplateChild::default(),
                stack: TemplateChild::default(),
                sidebar: TemplateChild::default(),
                leaflet: TemplateChild::default(),
                title_label: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            SidebarRow::static_type();
            CameraPage::static_type();
            WallpaperPage::static_type();
            DevicePage::static_type();
            LocationPage::static_type();
            NotificationPage::static_type();
            BackgroundPage::static_type();
            ScreenCastPage::static_type();
            AccountPage::static_type();
            NetworkMonitorPage::static_type();
            EmailPage::static_type();
            DocumentsPage::static_type();
            OpenUriPage::static_type();
            Self::bind_template(klass);

            klass.install_action("win.back", None, |win, _, _| {
                let self_ = imp::ExampleApplicationWindow::from_instance(win);
                self_.leaflet.navigate(adw::NavigationDirection::Back);
            });
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ExampleApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let builder = gtk::Builder::from_resource("/com/belmoussaoui/ashpd/demo/shortcuts.ui");
            let shortcuts = builder.object("shortcuts").unwrap();
            obj.set_help_overlay(Some(&shortcuts));

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
            let row = self.sidebar.row_at_index(0).unwrap();
            self.sidebar.unselect_row(&row);
            self.sidebar
                .connect_row_activated(clone!(@weak obj as win => move |_, row| {
                    win.sidebar_row_selected(row);
                }));
            self.stack.set_visible_child_name("welcome");
            // load latest window state
            obj.load_window_size();
        }
    }

    impl WidgetImpl for ExampleApplicationWindow {}
    impl WindowImpl for ExampleApplicationWindow {
        // save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            if let Err(err) = obj.save_window_size() {
                warn!("Failed to save window state, {}", &err);
            }
            Inhibit(false)
        }
    }

    impl ApplicationWindowImpl for ExampleApplicationWindow {}
    impl AdwApplicationWindowImpl for ExampleApplicationWindow {}
}

glib::wrapper! {
    pub struct ExampleApplicationWindow(ObjectSubclass<imp::ExampleApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl ExampleApplicationWindow {
    pub fn new(app: &ExampleApplication) -> Self {
        let window: Self =
            glib::Object::new(&[]).expect("Failed to create ExampleApplicationWindow");
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);

        window
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &imp::ExampleApplicationWindow::from_instance(self).settings;

        let size = self.default_size();

        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = &imp::ExampleApplicationWindow::from_instance(self).settings;

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    pub fn sidebar_row_selected(&self, row: &gtk::ListBoxRow) {
        let self_ = imp::ExampleApplicationWindow::from_instance(self);
        let sidebar_row = row.downcast_ref::<SidebarRow>().unwrap();
        self_.leaflet.navigate(adw::NavigationDirection::Forward);
        let page_name = sidebar_row.name();
        if self_.stack.child_by_name(&page_name).is_some() {
            self_.stack.set_visible_child_name(&page_name);
            self_.title_label.set_label(&sidebar_row.title().unwrap());
        } else {
            self_.title_label.set_label("");
            self_.stack.set_visible_child_name("welcome");
        }
    }
}
