// Copyright (C) 2024 GNOME Foundation
//
// Authors:
//     Hubert Figuière <hub@figuiere.net>
//

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use adw::{prelude::*, subclass::prelude::*};
use futures_util::{lock::Mutex, StreamExt};
use glib::clone;
use gtk::glib;

use ashpd::{
    desktop::{
        usb::{UsbOptions, UsbProxy},
        Session,
    },
    WindowIdentifier,
};

use crate::widgets::{PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/usb.ui")]
    pub struct UsbPage {
        #[template_child]
        pub usb_devices: TemplateChild<adw::PreferencesGroup>,
        rows: RefCell<HashMap<String, adw::PreferencesRow>>,
        pub session: Arc<Mutex<Option<Session<'static>>>>,
        pub event_source: Arc<Mutex<Option<glib::SourceId>>>,
    }

    impl UsbPage {
        fn add(&self, id: String, row: &impl IsA<adw::PreferencesRow>) {
            self.usb_devices.get().add(row.upcast_ref());
            self.rows.borrow_mut().insert(id, row.as_ref().clone());
        }

        fn clear_devices(&self) {
            for row in self.rows.borrow().iter() {
                self.usb_devices.get().remove(row.1);
            }
            self.rows.borrow_mut().clear();
        }

        pub(super) async fn refresh_devices(&self) -> ashpd::Result<()> {
            let page = self.obj();

            self.clear_devices();

            let usb = UsbProxy::new().await?;
            let devices = usb.enumerate_devices(UsbOptions::default()).await?;
            for device in devices {
                let row = adw::ActionRow::new();
                let vendor = device.1.vendor().unwrap_or_default();
                let dev = device.1.model().unwrap_or_default();
                row.set_title(&format!("{} {}", &vendor, &dev));
                if let Some(devnode) = device.1.device_file {
                    row.set_subtitle(&devnode);
                }
                let activatable =
                    gtk::Button::from_icon_name("preferences-system-sharing-symbolic");
                activatable.set_css_classes(&["circular"]);
                row.add_suffix(&activatable);
                row.add_prefix(&gtk::CheckButton::new());

                let device_id = device.0.clone();
                let device_writable = device.1.writable.unwrap_or(false);
                activatable.connect_clicked(move |row| {
                    glib::spawn_future_local(clone!(@strong row, @strong device_id => async move {
                        let root = row.native().unwrap();
                        let identifier = WindowIdentifier::from_native(&root).await;
                        let usb = UsbProxy::new().await.unwrap();
                        //                        if row.is_active() {
                        println!("acquire {device_id}");
                        let result = usb.acquire_devices(&identifier, &[
                            (&device_id, device_writable)
                        ]).await;
                        println!("result: {result:?}");
//                        } else {
//                            let _ = usb.release_devices(&[&device_id]).await;
//                        }
                    }));
                });
                page.imp().add(device.0.clone(), &row);
            }
            Ok(())
        }

        pub(super) async fn start_session(&self) -> ashpd::Result<()> {
            let usb = UsbProxy::new().await?;
            let session = usb.create_session().await?;
            self.session.lock().await.replace(session);

            let session = self.session.clone();
            glib::spawn_future(async move {
                let usb = UsbProxy::new().await?;
                loop {
                    if session.lock().await.is_none() {
                        tracing::debug!("session is gone");
                        break;
                    }
                    if let Some(response) = usb.receive_device_events().await?.next().await {
                        let events = response.events();
                        for ev in events {
                            println!(
                                "Received event: {} for device {}",
                                ev.event_action(),
                                ev.event_device_id()
                            );
                        }
                    }
                }
                tracing::debug!("Loop is gone");
                Ok::<(), ashpd::Error>(())
            });

            Ok(())
        }

        pub(super) async fn stop_session(&self) -> anyhow::Result<()> {
            if let Some(session) = self.session.lock().await.take() {
                session.close().await?;
            }
            Ok(())
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
                page.refresh_devices().await
            });
            klass.install_action_async("usb.start_session", None, |page, _, _| async move {
                page.start_session().await
            });
            klass.install_action_async("usb.stop_session", None, |page, _, _| async move {
                page.stop_session().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for UsbPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().action_set_enabled("usb.stop_session", false);
        }
    }

    impl WidgetImpl for UsbPage {
        fn map(&self) {
            glib::spawn_future_local(
                clone!(@weak self as widget => async move { widget.obj().refresh_devices().await; } ),
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

impl UsbPage {
    async fn refresh_devices(&self) {
        match self.imp().refresh_devices().await {
            Ok(_) => {}
            Err(err) => {
                tracing::error!("Failed to refresh USB devices: {err}");
                self.error("Failed to refresh USB devices.");
            }
        }
    }

    async fn start_session(&self) {
        self.action_set_enabled("usb.start_session", false);
        self.action_set_enabled("usb.stop_session", true);

        match self.imp().start_session().await {
            Ok(_) => self.info("USB session started"),
            Err(err) => {
                tracing::error!("Failed to start USB session: {err}");
                self.error("Failed to start USB session.");
                self.action_set_enabled("usb.start_session", true);
                self.action_set_enabled("usb.stop_session", false);
            }
        }
    }

    async fn stop_session(&self) {
        self.action_set_enabled("usb.start_session", true);
        self.action_set_enabled("usb.stop_session", false);

        match self.imp().stop_session().await {
            Ok(_) => self.info("USB session stopped"),
            Err(err) => {
                tracing::error!("Failed to stop USB session: {err}");
                self.error("Failed to stop USB session.");
            }
        }
    }
}
