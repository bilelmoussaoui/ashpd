use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use gtk::glib;

use ashpd::{
    desktop::usb::{UsbOptions, UsbProxy},
    WindowIdentifier,
};

use crate::widgets::{PortalPage, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/usb.ui")]
    pub struct UsbPage {
        #[template_child]
        pub usb_devices: TemplateChild<adw::PreferencesGroup>,
        rows: RefCell<Vec<adw::PreferencesRow>>,
    }

    impl UsbPage {
        fn add(&self, row: &impl IsA<adw::PreferencesRow>) {
            self.usb_devices.get().add(row.upcast_ref());
            self.rows.borrow_mut().push(row.as_ref().clone());
        }

        fn clear_devices(&self) {
            for row in self.rows.borrow().iter() {
                self.usb_devices.get().remove(row);
            }
            self.rows.borrow_mut().clear();
        }

        async fn refresh_devices(&self) {
            let page = self.obj();

            self.clear_devices();

            let usb = UsbProxy::new().await.unwrap();
            let devices = usb.enumerate_devices(UsbOptions::default()).await.unwrap();
            println!("devices {} {:?}", devices.len(), devices);
            for device in devices {
                let row = adw::ActionRow::new();
                let vendor = device.1.vendor().unwrap_or_default();
                let dev = device.1.model().unwrap_or_default();
                row.set_title(&format!("{} {}", &vendor, &dev));
                if let Some(devnode) = device.1.device_file {
                    row.set_subtitle(&devnode);
                }
                let activatable = gtk::Button::new();
                row.set_activatable_widget(Some(&activatable));

                let device_id = device.0.clone();
                let device_writable = device.1.writable.unwrap_or(false);
                row.connect_activated(move |row| {
                    glib::spawn_future_local(clone!(@strong row, @strong device_id => async move {
                        let root = row.native().unwrap();
                        let identifier = WindowIdentifier::from_native(&root).await;
                        let usb = UsbProxy::new().await.unwrap();
                        if row.is_active() {
                            usb.acquire_devices(&identifier, &[
                                (&device_id, device_writable)
                            ], true).await.map(|_| ());
                        } else {
                            usb.release_devices(&[&device_id]).await;
                        }
                        ()
                    }));
                });
                page.imp().add(&row);
            }
        }
    }
    #[glib::object_subclass]
    impl ObjectSubclass for UsbPage {
        const NAME: &'static str = "UsbPage";
        type Type = super::UsbPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("usb.refresh", None, |page, _, _| async move {
                page.imp().refresh_devices().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for UsbPage {}
    impl WidgetImpl for UsbPage {
        fn map(&self) {
            glib::spawn_future_local(
                clone!(@weak self as widget => async move { widget.refresh_devices().await; } ),
            );

            self.parent_map();
        }
    }
    impl BinImpl for UsbPage {}
    impl PortalPageImpl for UsbPage {}
}

glib::wrapper! {
    pub struct UsbPage(ObjectSubclass<imp::UsbPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl UsbPage {}
