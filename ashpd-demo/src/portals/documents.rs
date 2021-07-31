use ashpd::documents::DocumentsProxy;
use ashpd::zbus;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::config;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/documents.ui")]
    pub struct DocumentsPage {
        #[template_child]
        mount_point: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DocumentsPage {
        const NAME: &'static str = "DocumentsPage";
        type Type = super::DocumentsPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<adw::ClampLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for DocumentsPage {
        fn constructed(&self, _obj: &Self::Type) {
            let ctx = glib::MainContext::default();
            let mount_point_label = self.mount_point.get();
            ctx.spawn_local(clone!(@weak mount_point_label => async move {
                            let cnx = zbus::azync::Connection::session().await.unwrap();
                            let proxy = DocumentsProxy::new(&cnx).await.unwrap();
                            let info = proxy.list(config::APP_ID).await;
                            println!("{:#?}", info);
                            let mount_point = proxy.mount_point().await;
                            println!("{:#?}", mount_point);
            //                mount_point_label.set_text(&mount_point);
                        }));
        }
    }

    impl WidgetImpl for DocumentsPage {}
    impl BinImpl for DocumentsPage {}
}

glib::wrapper! {
    pub struct DocumentsPage(ObjectSubclass<imp::DocumentsPage>) @extends gtk::Widget, adw::Bin;
}

impl DocumentsPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a DocumentsPage")
    }
}
