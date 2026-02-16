use adw::{prelude::*, subclass::prelude::*};
use ashpd::{Uri, desktop::proxy_resolver::ProxyResolver};
use gtk::glib;

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/proxy_resolver.ui")]
    pub struct ProxyResolverPage {
        #[template_child]
        pub uri: TemplateChild<adw::EntryRow>,
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
    impl WidgetImpl for ProxyResolverPage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { ProxyResolver::new().await }).await {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for ProxyResolverPage {}
    impl PortalPageImpl for ProxyResolverPage {}
}

glib::wrapper! {
    pub struct ProxyResolverPage(ObjectSubclass<imp::ProxyResolverPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl ProxyResolverPage {
    async fn resolve(&self) -> ashpd::Result<()> {
        let imp = self.imp();

        match Uri::parse(&imp.uri.text()) {
            Ok(uri) => {
                let response = spawn_tokio(async move {
                    let proxy = ProxyResolver::new().await?;
                    proxy.lookup(&uri).await
                })
                .await;
                match response {
                    Ok(resolved_uris) => {
                        resolved_uris.iter().for_each(|uri| {
                            let row = adw::ActionRow::builder()
                                .title(uri.as_str())
                                .selectable(true)
                                .build();
                            imp.response_group.add(&row);
                        });
                        imp.response_group.set_visible(true);
                        self.success("Lookup request was successful");
                    }
                    Err(err) => {
                        tracing::error!("Failed to lookup URI: {err}");
                        self.error("Request to lookup a URI failed");
                    }
                }
            }
            Err(err) => {
                tracing::error!("Failed to parse URI: {err}");
                self.error("Request to lookup a URI failed");
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
