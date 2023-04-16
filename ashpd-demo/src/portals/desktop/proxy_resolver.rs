use adw::{prelude::*, subclass::prelude::*};
use ashpd::desktop::proxy_resolver::ProxyResolver;
use gtk::glib;

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/proxy_resolver.ui")]
    pub struct ProxyResolverPage {
        #[template_child]
        pub uri: TemplateChild<gtk::Entry>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyResolverPage {
        const NAME: &'static str = "ProxyResolverPage";
        type Type = super::ProxyResolverPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("proxy_resolver.resolve", None, |page, _, _| async move {
                if let Err(err) = page.resolve().await {
                    tracing::error!("Failed to resolve proxy {}", err);
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ProxyResolverPage {}
    impl WidgetImpl for ProxyResolverPage {}
    impl BinImpl for ProxyResolverPage {}
    impl PortalPageImpl for ProxyResolverPage {}
}

glib::wrapper! {
    pub struct ProxyResolverPage(ObjectSubclass<imp::ProxyResolverPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl ProxyResolverPage {
    async fn resolve(&self) -> ashpd::Result<()> {
        let imp = self.imp();
        let proxy = ProxyResolver::new().await?;

        match url::Url::parse(&imp.uri.text()) {
            Ok(uri) => match proxy.lookup(&uri).await {
                Ok(resolved_uris) => {
                    resolved_uris.iter().for_each(|uri| {
                        let row = adw::ActionRow::builder()
                            .title(uri.as_str())
                            .selectable(true)
                            .build();
                        imp.response_group.add(&row);
                    });
                    imp.response_group.set_visible(true);
                    self.send_notification(
                        "Lookup request was successful",
                        NotificationKind::Success,
                    );
                }
                Err(_err) => {
                    self.send_notification(
                        "Request to lookup a URI failed",
                        NotificationKind::Error,
                    );
                }
            },
            Err(_err) => {
                self.send_notification("Request to lookup a URI failed", NotificationKind::Error);
            }
        };

        Ok(())
    }
}

impl Default for ProxyResolverPage {
    fn default() -> Self {
        glib::Object::new()
    }
}
