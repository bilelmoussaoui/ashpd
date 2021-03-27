use ashpd::desktop::network_monitor::NetworkMonitorProxy;
use ashpd::zbus;
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/network_monitor.ui")]
    pub struct NetworkMonitorPage {
        #[template_child]
        pub network_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub metered: TemplateChild<gtk::Label>,
        #[template_child]
        pub connectivity: TemplateChild<gtk::Label>,
        pub connection: zbus::Connection,
    }

    impl Default for NetworkMonitorPage {
        fn default() -> Self {
            Self {
                network_available: TemplateChild::default(),
                metered: TemplateChild::default(),
                connectivity: TemplateChild::default(),
                connection: zbus::Connection::new_session().unwrap(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NetworkMonitorPage {
        const NAME: &'static str = "NetworkMonitorPage";
        type Type = super::NetworkMonitorPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "network_monitor.lookup",
                None,
                move |page, _action, _target| {
                    //page.pick_color().unwrap();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NetworkMonitorPage {
        fn constructed(&self, obj: &Self::Type) {
            let proxy = NetworkMonitorProxy::new(&self.connection).unwrap();
            let network_status = proxy.get_status().unwrap();

            self.network_available
                .set_text(&network_status.available.to_string());
            self.metered.set_text(&network_status.metered.to_string());
            self.connectivity
                .set_text(&network_status.connectivity.to_string());
        }
    }
    impl WidgetImpl for NetworkMonitorPage {}
    impl BoxImpl for NetworkMonitorPage {}
}

glib::wrapper! {
    pub struct NetworkMonitorPage(ObjectSubclass<imp::NetworkMonitorPage>) @extends gtk::Widget, gtk::Box;
}

impl NetworkMonitorPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a NetworkMonitorPage")
    }
}
