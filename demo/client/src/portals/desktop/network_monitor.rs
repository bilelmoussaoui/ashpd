use adw::{prelude::*, subclass::prelude::*};
use ashpd::desktop::network_monitor::NetworkMonitor;
use gtk::glib::{self, clone};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/network_monitor.ui")]
    pub struct NetworkMonitorPage {
        #[template_child]
        pub network_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub metered: TemplateChild<gtk::Label>,
        #[template_child]
        pub connectivity: TemplateChild<gtk::Label>,
        #[template_child]
        pub host_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub port_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub can_reach_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NetworkMonitorPage {
        const NAME: &'static str = "NetworkMonitorPage";
        type Type = super::NetworkMonitorPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action_async(
                "network_monitor.can_reach",
                None,
                |page, _, _| async move {
                    if let Err(err) = page.can_reach().await {
                        tracing::error!("Failed to call can reach on NetworkMonitor {}", err);
                        page.error(&format!(
                            "Failed to call can reach on NetworkMonitor: {err}"
                        ));
                    }
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NetworkMonitorPage {}
    impl WidgetImpl for NetworkMonitorPage {
        fn map(&self) {
            let widget = self.obj();
            glib::spawn_future_local(clone!(
                #[weak]
                widget,
                async move {
                    if let Err(err) = widget.refresh().await {
                        tracing::error!("Failed to call can refresh on NetworkMonitor {}", err);
                        widget.error(&format!(
                            "Failed to call can refresh on NetworkMonitor: {err}"
                        ));
                    }
                }
            ));

            glib::spawn_future_local(glib::clone!(
                #[weak]
                widget,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { NetworkMonitor::new().await }).await {
                        widget.set_property("portal-version", proxy.version());
                    }
                }
            ));
            self.parent_map();
        }
    }
    impl BinImpl for NetworkMonitorPage {}
    impl PortalPageImpl for NetworkMonitorPage {}
}

glib::wrapper! {
    pub struct NetworkMonitorPage(ObjectSubclass<imp::NetworkMonitorPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl NetworkMonitorPage {
    async fn refresh(&self) -> ashpd::Result<()> {
        let imp = self.imp();
        let status = spawn_tokio(async move {
            let proxy = NetworkMonitor::new().await?;
            proxy.status().await
        })
        .await?;

        match status.connectivity() {
            ashpd::desktop::network_monitor::Connectivity::Local => {
                imp.connectivity.set_label("Local")
            }
            ashpd::desktop::network_monitor::Connectivity::Limited => {
                imp.connectivity.set_label("Limited")
            }
            ashpd::desktop::network_monitor::Connectivity::CaptivePortal => {
                imp.connectivity.set_label("Captive Portal")
            }
            ashpd::desktop::network_monitor::Connectivity::FullNetwork => {
                imp.connectivity.set_label("Full Network")
            }
        }
        imp.network_available
            .set_label(if status.is_available() { "Yes" } else { "No" });
        imp.metered
            .set_label(if status.is_metered() { "Yes" } else { "No" });

        Ok(())
    }

    async fn can_reach(&self) -> ashpd::Result<()> {
        let imp = self.imp();
        let hostname = imp.host_entry.text();
        let port = imp.port_entry.text().parse().unwrap_or(80);

        let response = spawn_tokio(async move {
            let proxy = NetworkMonitor::new().await?;
            proxy.can_reach(&hostname, port).await
        })
        .await;

        match response {
            Ok(response) => {
                imp.can_reach_row
                    .set_title(if response { "Yes" } else { "No" });
                imp.response_group.set_visible(true);
                self.success("Can reach request was successful");
            }
            Err(err) => {
                tracing::error!("Can reach request failed: {err}");
                self.error(&format!("Request failed: {err}"));
            }
        }

        Ok(())
    }
}

impl Default for NetworkMonitorPage {
    fn default() -> Self {
        glib::Object::new()
    }
}
