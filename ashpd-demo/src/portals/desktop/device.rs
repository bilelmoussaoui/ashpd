use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};
use ashpd::{
    desktop::device::{Device, DeviceProxy},
    zbus,
};
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/device.ui")]
    pub struct DevicePage {
        #[template_child]
        pub camera_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub microphone_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub speakers_switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DevicePage {
        const NAME: &'static str = "DevicePage";
        type Type = super::DevicePage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("device.request", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    match page.request().await {
                        Ok(_) => {
                            page.send_notification("Device access request was successful", NotificationKind::Success);
                        }
                        Err(err) => {
                            tracing::error!("Failed to request device access {}", err);
                            page.send_notification("Request to access a device failed", NotificationKind::Error);
                        }
                    }
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for DevicePage {
        fn constructed(&self, obj: &Self::Type) {
            obj.set_sensitive(!ashpd::is_sandboxed());
        }
    }
    impl WidgetImpl for DevicePage {}
    impl BinImpl for DevicePage {}
    impl PortalPageImpl for DevicePage {}
}

glib::wrapper! {
    pub struct DevicePage(ObjectSubclass<imp::DevicePage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl DevicePage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a DevicePage")
    }

    async fn request(&self) -> ashpd::Result<()> {
        let cnx = zbus::Connection::session().await?;
        let proxy = DeviceProxy::new(&cnx).await?;

        proxy
            .access_device(std::process::id(), self.selected_devices().as_slice())
            .await?;

        Ok(())
    }

    /// Returns the selected Devices
    fn selected_devices(&self) -> Vec<Device> {
        let self_ = imp::DevicePage::from_instance(self);

        let mut devices = Vec::new();
        if self_.speakers_switch.is_active() {
            devices.push(Device::Speakers);
        }
        if self_.camera_switch.is_active() {
            devices.push(Device::Camera);
        }
        if self_.microphone_switch.is_active() {
            devices.push(Device::Microphone);
        }
        devices
    }
}
