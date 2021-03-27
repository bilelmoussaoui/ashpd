use ashpd::desktop::screenshot::{Color, PickColorOptions, ScreenshotProxy};
use ashpd::zbus;
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screenshot.ui")]
    pub struct ScreenshotPage {
        pub request: RefCell<Option<zbus::Connection>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotPage {
        const NAME: &'static str = "ScreenshotPage";
        type Type = super::ScreenshotPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "screenshot.pick-color",
                None,
                move |page, _action, _target| {
                    page.pick_color().unwrap();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenshotPage {}
    impl WidgetImpl for ScreenshotPage {}
    impl BoxImpl for ScreenshotPage {}
}

glib::wrapper! {
    pub struct ScreenshotPage(ObjectSubclass<imp::ScreenshotPage>) @extends gtk::Widget, gtk::Box;
}

impl ScreenshotPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a ScreenshotPage")
    }

    pub fn pick_color(&self) -> zbus::fdo::Result<()> {
        let self_ = imp::ScreenshotPage::from_instance(self);

        let connection = zbus::Connection::new_session()?;
        let proxy = ScreenshotProxy::new(&connection)?;
        let request = proxy.pick_color(WindowIdentifier::default(), PickColorOptions::default())?;
        println!("{:#?}", request.path());
        request.connect_response(|response: Response<Color>| {
            println!("{:#?}", response);
            if let Response::Ok(color) = response {
                println!("({}, {}, {})", color.red(), color.green(), color.blue());
            }
            Ok(())
        })?;
        self_.request.replace(Some(connection));
        Ok(())
    }
}
