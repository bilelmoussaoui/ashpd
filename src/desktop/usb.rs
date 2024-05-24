// Copyright (C) 2024 GNOME Foundation
//
// Authors:
//     Hubert Figuière <hub@figuiere.net>
//

//! Provide an interface to USB device. Allow enumerating devices
//! and requiring access to.
//! ```rust,no_run
//! use ashpd::desktop::usb::UsbProxy;
//! use futures_util::StreamExt;
//!
//! async fn watch_devices() -> ashpd::Result<()> {
//!     let usb = UsbProxy::new().await?;
//!     let session = usb.create_session().await?;
//!     if let Some(response) = usb.receive_device_events().await?.next().await {
//!         let events = response.events();
//!         for ev in events {
//!              println!(
//!                  "Received event: {} for device {}",
//!                  ev.event_action(),
//!                  ev.event_device_id()
//!              );
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use futures_util::Stream;
use serde::Deserialize;
use zbus::zvariant::{
    DeserializeDict, Fd, ObjectPath, OwnedFd, OwnedObjectPath, OwnedValue, SerializeDict, Type,
    Value,
};

use super::Request;
use crate::{
    desktop::{HandleToken, Session},
    proxy::Proxy,
    Error, WindowIdentifier,
};

#[derive(Debug, SerializeDict, Type, Default)]
#[zvariant(signature = "dict")]
struct CreateSessionOptions {
    session_handle_token: HandleToken,
}

#[derive(Debug, SerializeDict, Type, Default)]
#[zvariant(signature = "dict")]
struct AcquireDevicesOptions {
    writable: bool,
    handle_token: HandleToken,
}

/// Options for the USB portal.
#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct UsbOptions {
    handle_token: HandleToken,
    session_handle_token: HandleToken,
}

/// USB device description
#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct UsbDevice {
    parent: Option<String>,
    readable: Option<bool>,
    /// Device is writable. Default is false.
    pub writable: Option<bool>,
    /// The device node for the USB.
    #[zvariant(rename = "device-file")]
    pub device_file: Option<String>,
    /// Device properties
    pub properties: Option<HashMap<String, OwnedValue>>,
}

impl UsbDevice {
    /// Return the vendor string for display.
    pub fn vendor(&self) -> Option<String> {
        self.properties.as_ref().and_then(|properties| {
            properties
                .get("ID_VENDOR_FROM_DATABASE")
                .or_else(|| properties.get("ID_VENDOR_ENC"))
                .and_then(|v| v.downcast_ref::<String>().ok())
        })
    }

    /// Return the model string for display.
    pub fn model(&self) -> Option<String> {
        self.properties.as_ref().and_then(|properties| {
            properties
                .get("ID_MODEL_FROM_DATABASE")
                .or_else(|| properties.get("ID_MODEL_ENC"))
                .and_then(|v| v.downcast_ref::<String>().ok())
        })
    }
}

/// Device to acquire.
#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct AcquireDevice {
    writable: bool,
}

/// Option for AcquireDevice call.
#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct AcquireOptions {
    handle_token: HandleToken,
}

/// Finished device acquired.
#[derive(DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct AcquiredDevice {
    success: bool,
    fd: Option<OwnedFd>,
    error: Option<String>,
}

impl AcquiredDevice {
    /// Return if the acquisition is a success.
    pub fn success(&self) -> bool {
        self.success
    }

    /// Return the error if it exists.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Return the open file descriptor
    pub fn fd(&self) -> Option<Fd> {
        self.fd.as_ref().map(|fd| fd.into())
    }
}

#[derive(Debug, Deserialize, Type)]
/// A usb event received part of the `device_event` signal response.
pub struct UsbEvent(/* action */ String, /* id */ String, UsbDevice);

#[derive(Debug, Deserialize, Type)]
/// A response received when the `device_event` signal is received.
pub struct UsbDeviceEvent(OwnedObjectPath, Vec<UsbEvent>);

impl UsbDeviceEvent {
    /// The session triggered the state change
    pub fn session_handle(&self) -> ObjectPath<'_> {
        self.0.as_ref()
    }

    /// Events received
    pub fn events(&self) -> &[UsbEvent] {
        &self.1
    }
}

impl UsbEvent {
    /// The action.
    pub fn event_action(&self) -> &str {
        &self.0
    }

    /// The device id.
    pub fn event_device_id(&self) -> &str {
        &self.1
    }

    /// The [`UsbDevice`] properties.
    pub fn event_device(&self) -> &UsbDevice {
        &self.2
    }
}

/// This interface provides access to USB devices.
///
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Usb")]
pub struct UsbProxy<'a>(Proxy<'a>);

impl<'a> UsbProxy<'a> {
    /// Create a new instance of [`UsbProxy`].
    pub async fn new() -> Result<UsbProxy<'a>, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.Usb").await?;
        Ok(Self(proxy))
    }

    /// Create Session
    ///
    #[doc(alias = "CreateSession")]
    pub async fn create_session(&self) -> Result<Session<'a>, Error> {
        let options = CreateSessionOptions::default();
        let session: OwnedObjectPath = self.0.call("CreateSession", &(&options)).await?;
        Session::new(session).await
    }

    /// Enumerate USB devices.
    ///
    #[doc(alias = "EnumerateDevices")]
    pub async fn enumerate_devices(
        &self,
        options: UsbOptions,
    ) -> Result<Vec<(String, UsbDevice)>, Error> {
        self.0.call("EnumerateDevices", &(&options)).await
    }

    /// Acquire devices
    ///
    /// The portal will perform the permission request.
    ///
    #[doc(alias = "AcquireDevices")]
    pub async fn acquire_devices(
        &self,
        parent_window: &WindowIdentifier,
        devices: &[(&String, bool)],
    ) -> Result<Request<()>, Error> {
        let options = AcquireOptions::default();
        let acquire_devices: Vec<(String, AcquireDevice)> = devices
            .iter()
            .map(|dev| {
                let device = AcquireDevice { writable: dev.1 };
                (dev.0.to_string(), device)
            })
            .collect();
        self.0
            .empty_request(
                &options.handle_token,
                "AcquireDevices",
                &(parent_window, &acquire_devices, &options),
            )
            .await
    }

    /// Call on success of acquire_devices.
    #[doc(alias = "FinishAcquireDevices")]
    pub async fn finish_acquire_devices(
        &self,
    ) -> Result<(Vec<(String, AcquiredDevice)>, bool), Error> {
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        self.0.call("FinishAcquireDevices", &(&options)).await
    }

    /// Release devices
    #[doc(alias = "ReleaseDevices")]
    pub async fn release_devices(&self, devices: &[&str]) -> Result<(), Error> {
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        self.0.call("ReleaseDevices", &(devices, &options)).await
    }

    /// Signal emitted on a device event
    #[doc(alias = "DeviceEvents")]
    pub async fn receive_device_events(&self) -> Result<impl Stream<Item = UsbDeviceEvent>, Error> {
        self.0.signal("DeviceEvents").await
    }
}
