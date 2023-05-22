use adw::subclass::prelude::*;
use ashpd::desktop::device::{Device, DeviceProxy};
use gtk::glib;

use crate::widgets::{PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
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
            klass.bind_template();

            klass.install_action_async("device.request", None, |page, _, _| async move {
                match page.request().await {
                    Ok(_) => {
                        page.success("Device access request was successful");
                    }
                    Err(err) => {
                        tracing::error!("Failed to request device access: {err}");
                        page.error("Request to access a device failed");
                    }
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for DevicePage {}
    impl WidgetImpl for DevicePage {}
    impl BinImpl for DevicePage {}
    impl PortalPageImpl for DevicePage {}
}

glib::wrapper! {
    pub struct DevicePage(ObjectSubclass<imp::DevicePage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl DevicePage {
    async fn request(&self) -> ashpd::Result<()> {
        let proxy = DeviceProxy::new().await?;

        proxy
            .access_device(std::process::id(), self.selected_devices().as_slice())
            .await?;

        Ok(())
    }

    /// Returns the selected Devices
    fn selected_devices(&self) -> Vec<Device> {
        let imp = self.imp();

        let mut devices = Vec::new();
        if imp.speakers_switch.is_active() {
            devices.push(Device::Speakers);
        }
        if imp.camera_switch.is_active() {
            devices.push(Device::Camera);
        }
        if imp.microphone_switch.is_active() {
            devices.push(Device::Microphone);
        }
        devices
    }
}

impl Default for DevicePage {
    fn default() -> Self {
        glib::Object::new()
    }
}
