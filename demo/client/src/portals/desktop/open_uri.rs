use std::os::fd::AsFd;

use adw::subclass::prelude::*;
use ashpd::desktop::open_uri;
use ashpd_handles::WindowIdentifier;
use gtk::{glib, prelude::*};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/open_uri.ui")]
    pub struct OpenUriPage {
        #[template_child]
        pub writeable_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub ask_switch: TemplateChild<adw::SwitchRow>,
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
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
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
                let response = spawn_tokio(async move {
                    let request = open_uri::OpenFileRequest::default()
                        .ask(ask)
                        .writeable(writeable)
                        .identifier(identifier);
                    if uri.scheme() == "file" {
                        let file_path = uri.to_file_path().unwrap();
                        match std::fs::File::open(&file_path) {
                            Ok(fd) => request.send_file(&fd.as_fd()).await,
                            Err(err) => {
                                tracing::error!("Failed to open file: {err}");
                                return Err(From::from(err));
                            }
                        }
                    } else {
                        request.send_uri(&uri).await
                    }
                    .and_then(|r| r.response())
                })
                .await;

                match response {
                    Ok(()) => {
                        self.success("Open URI request was successful");
                    }
                    Err(err) => {
                        tracing::error!("Failed to open URI: {err}");
                        self.error("Request to open URI failed");
                    }
                }
            }
            Err(err) => {
                tracing::error!("Failed to parse URI: {err}");
                self.error("Malformed URI");
            }
        }
    }
}
