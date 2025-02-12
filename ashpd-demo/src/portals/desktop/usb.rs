// Copyright (C) 2024-2025 GNOME Foundation
//
// Authors:
//     Hubert Figuière <hub@figuiere.net>
//

use std::{collections::HashMap, os::fd::AsRawFd, sync::Arc};

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    desktop::{
        usb::{Device, UsbDevice, UsbError, UsbProxy},
        Session,
    },
    zbus::zvariant::{Fd, OwnedFd},
    WindowIdentifier,
};
use futures_util::{lock::Mutex, StreamExt};
use glib::clone;
use gtk::glib;
use rusb::UsbContext;

use crate::widgets::{PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/usb.ui")]
    pub struct UsbPage {
        #[template_child]
        pub usb_devices: TemplateChild<adw::PreferencesGroup>,
        rows: RefCell<HashMap<String, super::row::UsbDeviceRow>>,
        pub session: Arc<Mutex<Option<Session<'static, UsbProxy<'static>>>>>,
        pub event_source: Arc<Mutex<Option<glib::SourceId>>>,
    }

    impl UsbPage {
        fn add(&self, uuid: String, row: super::row::UsbDeviceRow) {
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

        pub(super) fn released_device(&self, uuid: &str) {
            if let Some(row) = self.rows.borrow().get(uuid) {
                row.release();
            }
        }

        fn usb_describe_device(fd: &dyn AsRawFd) -> ashpd::Result<String> {
            let context = rusb::Context::new()
                .map_err(|_| ashpd::PortalError::Failed("rusb Context".to_string()))?;
            let handle = unsafe { context.open_device_with_fd(fd.as_raw_fd()) }
                .map_err(|_| ashpd::PortalError::Failed("open USB device".to_string()))?;
            let device = handle.device();
            let device_desc = device.device_descriptor().unwrap();
            Ok(format!(
                "Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id()
            ))
        }

        fn add_device_row(&self, page: &super::UsbPage, device: &(String, UsbDevice)) {
            let row = super::row::UsbDeviceRow::with_device(
                page.clone(),
                device.0.clone(),
                device.1.is_writable(),
            );
            let vendor = device.1.vendor().unwrap_or_default();
            let dev = device.1.model().unwrap_or_default();
            row.set_title(&format!("{} {}", &vendor, &dev));
            if let Some(devnode) = &device.1.device_file() {
                row.set_subtitle(devnode);
            }
            page.imp().add(device.0.clone(), row);
        }

        pub(super) async fn refresh_devices(&self) -> ashpd::Result<()> {
            let page = self.obj();

            self.clear_devices();

            let usb = UsbProxy::new().await?;
            let devices = usb.enumerate_devices().await?;
            for device in devices {
                self.add_device_row(&page, &device);
            }
            Ok(())
        }

        pub(super) fn finish_acquire_devices(
            &self,
            devices: &[(String, Result<OwnedFd, UsbError>)],
        ) {
            devices.iter().for_each(|device| {
                if let Ok(fd) = &device.1 {
                    match Self::usb_describe_device(&Fd::from(fd)) {
                        Ok(describe) => self.obj().info(&describe),
                        Err(err) => self.obj().info(&err.to_string()),
                    }
                }
                self.acquired_device(&device.0);
            });
        }

        pub(super) async fn start_session(&self) -> ashpd::Result<()> {
            let usb = UsbProxy::new().await?;
            let session = usb.create_session().await?;
            self.session.lock().await.replace(session);

            let session = self.session.clone();
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
                            ev.action(),
                            ev.device_id()
                        );
                    }
                }
            }
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
            glib::spawn_future_local(clone!(
                #[weak(rename_to = widget)]
                self,
                async move {
                    widget.obj().refresh_devices().await;
                }
            ));

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
                self.error(&format!("Failed to refresh USB devices: {err}."));
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
                self.error(&format!("Failed to start USB session: {err}."));
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
                self.error(&format!("Failed to stop USB session: {err}."));
            }
        }
    }

    async fn do_share(&self, device_id: &String, device_writable: bool) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let usb = UsbProxy::new().await?;
        let devices = usb
            .acquire_devices(
                identifier.as_ref(),
                &[Device::new(device_id.to_string(), device_writable)],
            )
            .await?;

        self.imp().finish_acquire_devices(&devices);
        Ok(())
    }

    async fn share(&self, device_id: &String, device_writable: bool) {
        let result = self.do_share(device_id, device_writable).await;
        if let Err(err) = result {
            tracing::error!("Acquire device error: {err}");
            self.error(&format!("Acquire device error: {err}"));
        }
    }

    async fn unshare(&self, device_id: &str) {
        let result = async {
            let usb = UsbProxy::new().await?;
            usb.release_devices(&[device_id]).await
        }
        .await;
        if let Err(err) = result {
            tracing::error!("Acquire device error: {err}");
            self.error(&format!("Acquire device error: {err}"));
        }
        self.imp().released_device(device_id);
    }
}

mod row {
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::glib;

    use super::UsbPage;

    mod imp {
        use std::cell::{Cell, RefCell};

        use adw::subclass::prelude::*;
        use gtk::glib;

        #[derive(Debug, gtk::CompositeTemplate, Default)]
        #[template(resource = "/com/belmoussaoui/ashpd/demo/usb_device_row.ui")]
        pub struct UsbDeviceRow {
            #[template_child]
            pub(super) checkbox: TemplateChild<gtk::CheckButton>,
            #[template_child]
            pub(super) acquire: TemplateChild<gtk::Button>,
            #[template_child]
            pub(super) release: TemplateChild<gtk::Button>,
            pub(super) page: RefCell<Option<super::UsbPage>>,
            pub(super) device_id: RefCell<String>,
            pub(super) writable: Cell<bool>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for UsbDeviceRow {
            const NAME: &'static str = "UsbDeviceRow";
            type Type = super::UsbDeviceRow;
            type ParentType = adw::ActionRow;

            fn class_init(klass: &mut Self::Class) {
                klass.bind_template();
                klass.bind_template_callbacks();
            }

            fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
                obj.init_template();
            }
        }

        #[gtk::template_callbacks]
        impl UsbDeviceRow {
            #[template_callback]
            async fn handle_acquire_clicked(&self, _: &gtk::Button) {
                let page = { self.page.borrow().clone() };
                if let Some(page) = page {
                    let device_id = self.device_id.borrow().clone();
                    let writable = self.writable.get();
                    page.share(&device_id, writable).await;
                }
            }

            #[template_callback]
            async fn handle_release_clicked(&self, _: &gtk::Button) {
                let page = { self.page.borrow().clone() };
                if let Some(page) = page {
                    let device_id = self.device_id.borrow().clone();
                    page.unshare(&device_id).await;
                }
            }
        }

        impl ObjectImpl for UsbDeviceRow {}
        impl WidgetImpl for UsbDeviceRow {}
        impl ListBoxRowImpl for UsbDeviceRow {}
        impl PreferencesRowImpl for UsbDeviceRow {}
        impl ActionRowImpl for UsbDeviceRow {}
    }

    glib::wrapper! {
        pub struct UsbDeviceRow(ObjectSubclass<imp::UsbDeviceRow>)
            @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
    }

    impl UsbDeviceRow {
        pub(super) fn with_device(page: UsbPage, device_id: String, writable: bool) -> Self {
            let obj: Self = glib::Object::new();

            let imp = obj.imp();
            imp.page.replace(Some(page));
            imp.device_id.replace(device_id);
            imp.writable.set(writable);
            obj
        }

        pub(super) fn acquire(&self) {
            self.imp().checkbox.set_active(true);
        }

        pub(super) fn release(&self) {
            self.imp().checkbox.set_active(false);
        }
    }
}
