use ashpd::zbus;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

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
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "network_monitor.lookup",
                None,
                move |_page, _action, _target| {
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
            obj.init();
        }
    }
    impl WidgetImpl for NetworkMonitorPage {}
    impl BinImpl for NetworkMonitorPage {}
}

glib::wrapper! {
    pub struct NetworkMonitorPage(ObjectSubclass<imp::NetworkMonitorPage>) @extends gtk::Widget, adw::Bin;
}

impl NetworkMonitorPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a NetworkMonitorPage")
    }

    fn init(&self) {
        /*
        let proxy = NetworkMonitorProxy::new(&self.connection).unwrap();
        obj.set_sensitive(!ashpd::is_sandboxed());

        // This portal is not available inside a sandbox
        if !ashpd::is_sandboxed() {
            self.network_available
                .set_text(&proxy.get_available().unwrap().to_string());
            self.metered
                .set_text(&proxy.get_metered().unwrap().to_string());
            self.connectivity
                .set_text(&proxy.get_connectivity().unwrap().to_string());
        }
         */
    }
}
