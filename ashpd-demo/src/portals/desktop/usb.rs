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

const SHARE_ICON: &str = "preferences-system-sharing-symbolic";
const UNSHARE_ICON: &str = "view-restore-symbolic";

glib::wrapper! {
    pub struct DeviceRow(ObjectSubclass<imp::DeviceRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl DeviceRow {
    fn new() -> Self {
        glib::Object::new()
    }

    fn is_active(&self) -> bool {
        self.imp().checkbox.is_active()
    }

    fn acquire(&self) {
        self.imp().checkbox.set_active(true);
        self.imp().acquire.set_icon_name(UNSHARE_ICON);
    }

    fn release(&self) {
        self.imp().checkbox.set_active(false);
        self.imp().acquire.set_icon_name(SHARE_ICON);
    }

    fn connect_share_clicked<F: Fn(&gtk::Button) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.imp().acquire.connect_clicked(f)
    }
}

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/usbdevicerow.ui")]
    pub struct DeviceRow {
        #[template_child]
        pub(super) checkbox: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub(super) acquire: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DeviceRow {
        const NAME: &'static str = "DeviceRow";
        type Type = super::DeviceRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DeviceRow {}
    impl WidgetImpl for DeviceRow {}
    impl ListBoxRowImpl for DeviceRow {}
    impl PreferencesRowImpl for DeviceRow {}
    impl ActionRowImpl for DeviceRow {}

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/usb.ui")]
    pub struct UsbPage {
        #[template_child]
        pub usb_devices: TemplateChild<adw::PreferencesGroup>,
        rows: RefCell<HashMap<String, super::DeviceRow>>,
        pub session: Arc<Mutex<Option<Session<'static>>>>,
        pub event_source: Arc<Mutex<Option<glib::SourceId>>>,
    }

    impl UsbPage {
        fn add(&self, uuid: String, row: super::DeviceRow) {
            self.usb_devices.get().add(&row);
            self.rows.borrow_mut().insert(uuid, row);
        }

        fn clear_devices(&self) {
            for row in self.rows.borrow().values() {
                self.usb_devices.get().remove(row);
            }
            self.rows.borrow_mut().clear();
        }

        fn acquired_device(&self, uuid: &str) {
            if let Some(row) = self.rows.borrow().get(uuid) {
                row.acquire();
            }
        }

        fn released_device(&self, uuid: &str) {
            if let Some(row) = self.rows.borrow().get(uuid) {
                row.release();
            }
        }

        pub(super) async fn refresh_devices(&self) -> ashpd::Result<()> {
            let page = self.obj();

            self.clear_devices();

            let usb = UsbProxy::new().await?;
            let devices = usb.enumerate_devices(UsbOptions::default()).await?;
            for device in devices {
                let row = super::DeviceRow::new();
                let vendor = device.1.vendor().unwrap_or_default();
                let dev = device.1.model().unwrap_or_default();
                row.set_title(&format!("{} {}", &vendor, &dev));
                if let Some(devnode) = device.1.device_file {
                    row.set_subtitle(&devnode);
                }

                let device_id = device.0.clone();
                let device_writable = device.1.writable.unwrap_or(false);
                row.connect_share_clicked(clone!(@strong page => move |row| {
                    glib::spawn_future_local(clone!(@strong row, @strong device_id, @strong page => async move {
                        let root = row.native().unwrap();
                        let identifier = WindowIdentifier::from_native(&root).await;
                        let usb = UsbProxy::new().await.unwrap();
                        let active = page.imp().rows.borrow().get(&device_id).map(|row| row.is_active()).unwrap_or(false);
                        if !active {
                            let result = usb.acquire_devices(&identifier, &[
                                (&device_id, device_writable)
                            ]).await;
                            match result {
                                Ok(_) => {
                                    loop {
                                        let result = usb.finish_acquire_devices().await;
                                        match result {
                                            Ok(result) => {
                                                println!("result {result:?}");
                                                if !result.1 {
                                                    continue;
                                                }
                                                for device in &result.0 {
                                                    page.imp().acquired_device(&device.0);
                                                }
                                            }
                                            Err(err) => {
                                                tracing::error!("Finish acquire device error: {err}");
                                                page.error(&format!("Finish acquire device error: {err}"));
                                            }
                                        }
                                        break;
                                    }
                                },
                                Err(err) => {
                                    tracing::error!("Acquire device error: {err}");
                                    page.error(&format!("Acquire device error: {err}"));
                                }
                            }
                        } else {
                            let result = usb.release_devices(&[&device_id]).await;
                            println!("{result:?}");
                            match result {
                                Ok(_) => {
                                }
                                Err(err) => {
                                    tracing::error!("Acquire device error: {err}");
                                    page.error(&format!("Acquire device error: {err}"));
                                }
                            }
                            page.imp().released_device(&device_id);
                        }
                    }));
                }));
                page.imp().add(device.0.clone(), row);
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
