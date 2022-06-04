use ashpd::{documents::DocumentsProxy, zbus};
use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use crate::widgets::{PortalPage, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/documents.ui")]
    pub struct DocumentsPage {
        #[template_child]
        pub mount_point: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DocumentsPage {
        const NAME: &'static str = "DocumentsPage";
        type Type = super::DocumentsPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for DocumentsPage {}

    impl WidgetImpl for DocumentsPage {
        fn map(&self, widget: &Self::Type) {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak widget => async move {
                if let Err(err) = widget.refresh().await {
                    tracing::error!("Failed to call a method on Documents portal{}", err);
                }
            }));
            self.parent_map(widget);
        }
    }
    impl BinImpl for DocumentsPage {}
    impl PortalPageImpl for DocumentsPage {}
}

glib::wrapper! {
    pub struct DocumentsPage(ObjectSubclass<imp::DocumentsPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl DocumentsPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a DocumentsPage")
    }

    async fn refresh(&self) -> ashpd::Result<()> {
        let cnx = zbus::Connection::session().await?;
        let proxy = DocumentsProxy::new(&cnx).await?;

        let mount_point = proxy.mount_point().await?;
        self.imp()
            .mount_point
            .set_label(mount_point.to_str().unwrap());

        Ok(())
    }
}
