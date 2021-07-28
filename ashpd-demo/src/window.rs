use glib::signal::Inhibit;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use tracing::warn;

use crate::application::ExampleApplication;
use crate::config::APP_ID;
use crate::portals::desktop::{
    AccountPage, BackgroundPage, CameraPage, DevicePage, EmailPage, FileChooserPage, InhibitPage,
    LocationPage, NetworkMonitorPage, NotificationPage, OpenUriPage, ScreenCastPage,
    ScreenshotPage, SecretPage, WallpaperPage,
};
use crate::portals::DocumentsPage;

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/window.ui")]
    pub struct ExampleApplicationWindow {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub screenshot: TemplateChild<ScreenshotPage>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
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
                leaflet: TemplateChild::default(),
                title_label: TemplateChild::default(),
                camera: TemplateChild::default(),
                wallpaper: TemplateChild::default(),
                location: TemplateChild::default(),
                notification: TemplateChild::default(),
                screencast: TemplateChild::default(),
                account: TemplateChild::default(),
                email: TemplateChild::default(),
                file_chooser: TemplateChild::default(),
                open_uri: TemplateChild::default(),
                inhibit: TemplateChild::default(),
                secret: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
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
    /*
        <child>
            <object class="GtkStackPage">
            <property name="name">background</property>
            <property name="title">Background</property>
            <property name="child">
                <object class="BackgroundPage" id="background" />
            </property>
            </object>
        </child>
        <child>
            <object class="GtkStackPage">
            <property name="name">device</property>
            <property name="title">Device</property>
            <property name="child">
                <object class="DevicePage" id="device" />
            </property>
            </object>
        </child>
        <child>
            <object class="GtkStackPage">
            <property name="name">documents</property>
            <property name="title">Documents</property>
            <property name="child">
                <object class="DocumentsPage" id="documents" />
            </property>
            </object>
        </child>
        <child>
            <object class="GtkStackPage">
            <property name="name">network_monitor</property>
            <property name="title">Network Monitor</property>
            <property name="child">
                <object class="NetworkMonitorPage" id="network_monitor" />
            </property>
            </object>
        </child>
    */
    impl ObjectImpl for ExampleApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            // Add pages based on whether the app is sandboxed
            if ashpd::is_sandboxed() {
                self.stack
                    .add_titled(&BackgroundPage::new(), Some("background"), "Background");
            } else {
                self.stack
                    .add_titled(&DevicePage::new(), Some("device"), "Device");
                self.stack.add_titled(
                    &NetworkMonitorPage::new(),
                    Some("network_monitor"),
                    "Network Monitor",
                );
                self.stack
                    .add_titled(&DocumentsPage::new(), Some("documents"), "Documents");
            }

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
}
