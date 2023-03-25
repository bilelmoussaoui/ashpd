use adw::subclass::prelude::*;
use ashpd::{desktop::open_uri, WindowIdentifier};
use gtk::{glib, prelude::*};

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/open_uri.ui")]
    pub struct OpenUriPage {
        #[template_child]
        pub writeable_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub ask_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub uri_entry: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OpenUriPage {
        const NAME: &'static str = "OpenUriPage";
        type Type = super::OpenUriPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("open_uri.uri", None, |page, _, _| async move {
                page.open_uri().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for OpenUriPage {}
    impl WidgetImpl for OpenUriPage {}
    impl BinImpl for OpenUriPage {}
    impl PortalPageImpl for OpenUriPage {}
}

glib::wrapper! {
    pub struct OpenUriPage(ObjectSubclass<imp::OpenUriPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl OpenUriPage {
    async fn open_uri(&self) {
        let imp = self.imp();
        let writeable = imp.writeable_switch.is_active();
        let ask = imp.ask_switch.is_active();
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        match url::Url::parse(&imp.uri_entry.text()) {
            Ok(uri) => {
                let request = open_uri::OpenFileRequest::default()
                    .ask(ask)
                    .writeable(writeable)
                    .identifier(identifier);
                match request.send_uri(&uri).await {
                    Ok(_) => {
                        self.send_notification(
                            "Open URI request was successful",
                            NotificationKind::Success,
                        );
                    }
                    Err(_err) => {
                        self.send_notification(
                            "Request to open URI failed",
                            NotificationKind::Error,
                        );
                    }
                }
            }
            Err(_err) => {
                self.send_notification("Malformed URI", NotificationKind::Error);
            }
        }
    }
}
