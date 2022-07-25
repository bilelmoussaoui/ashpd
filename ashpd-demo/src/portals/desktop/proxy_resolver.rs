use adw::prelude::*;
use ashpd::{desktop::proxy_resolver::ProxyResolverProxy, zbus};
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
};

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

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
            Self::bind_template(klass);

            klass.install_action(
                "proxy_resolver.resolve",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        if let Err(err) = page.resolve().await {
                            tracing::error!("Failed to resolve proxy {}", err);
                        }
                    }));
                },
            );
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
    pub struct ProxyResolverPage(ObjectSubclass<imp::ProxyResolverPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl ProxyResolverPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a ProxyResolverPage")
    }

    async fn resolve(&self) -> ashpd::Result<()> {
        let imp = self.imp();
        let proxy = ProxyResolverProxy::new().await?;

        match url::Url::parse(&imp.uri.text()) {
            Ok(uri) => match proxy.lookup(&uri).await {
                Ok(resolved_uris) => {
                    resolved_uris.iter().for_each(|uri| {
                        let row = adw::ActionRow::builder().title(uri.as_str()).build();
                        imp.response_group.add(&row);
                    });
                    imp.response_group.show();
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
