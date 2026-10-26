use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    MaybeAppID, WindowIdentifierType,
    backend::{
        IfaceInner, Result,
        request::{Request, RequestImpl},
    },
    desktop::{HandleToken, request::Response, usb::UsbDevice},
    wire::{Optional, OwnedObjectPath, Type, as_variant::optional},
};

#[derive(Debug, Deserialize, Type)]
#[zbus(signature = "dict")]
pub struct AcquireDevicesOptions {}

#[derive(Debug, Serialize, Deserialize, Type)]
#[zbus(signature = "dict")]
pub struct AccessOptions {
    #[serde(default, with = "optional", skip_serializing_if = "Option::is_none")]
    writable: Option<bool>,
}

impl AccessOptions {
    pub fn new(is_writable: bool) -> Self {
        Self {
            writable: Some(is_writable),
        }
    }

    pub fn is_writable(&self) -> Option<bool> {
        self.writable
    }
}

#[async_trait]
pub trait UsbImpl: RequestImpl {
    #[doc(alias = "AcquireDevices")]
    async fn acquire_devices(
        &self,
        token: HandleToken,
        window_identifier: Option<WindowIdentifierType>,
        app_id: Option<MaybeAppID>,
        devices: Vec<(String, UsbDevice, AccessOptions)>,
        options: AcquireDevicesOptions,
    ) -> Result<Vec<(String, AccessOptions)>>;
}

pub(crate) struct UsbInterface {
    imp: Arc<dyn UsbImpl>,
    inner: IfaceInner,
}

impl UsbInterface {
    pub fn new(imp: Arc<dyn UsbImpl>, inner: IfaceInner) -> Self {
        Self { imp, inner }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Usb")]
impl UsbInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "AcquireDevices")]
    #[zbus(out_args("response", "results"))]
    async fn acquire_devices(
        &self,
        handle: OwnedObjectPath,
        window_identifier: Optional<WindowIdentifierType>,
        app_id: Optional<MaybeAppID>,
        devices: Vec<(String, UsbDevice, AccessOptions)>,
        options: AcquireDevicesOptions,
    ) -> Result<Response<Vec<(String, AccessOptions)>>> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Usb::AcquireDevices",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
            async move {
                imp.acquire_devices(
                    HandleToken::try_from(&handle).unwrap(),
                    window_identifier.into(),
                    app_id.into(),
                    devices,
                    options,
                )
                .await
            },
        )
        .await
    }
}
