use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use enumflags2::{bitflags, BitFlags};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::Response,
    zbus::SignalContext,
    zvariant::{OwnedObjectPath, SerializeDict, Type},
    AppID, PortalError,
};

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, PartialEq, Eq, Debug, Type)]
#[repr(u32)]
pub enum Activity {
    Forbid = 0,
    Allow = 1,
    AllowInstance = 2,
}

#[derive(Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct Background {
    result: Activity,
}

impl Background {
    pub fn new(activity: Activity) -> Self {
        Self { result: activity }
    }
}

#[derive(Serialize_repr, Copy, Clone, PartialEq, Eq, Debug, Type)]
#[repr(u32)]
pub enum AppState {
    Background = 0,
    Running = 1,
    Active = 2,
}

#[bitflags]
#[derive(Deserialize_repr, PartialEq, Eq, Copy, Clone, Debug, Type)]
#[repr(u32)]
pub enum AutoStartFlags {
    DBusActivation = 1,
}

#[async_trait]
pub trait BackgroundImpl: RequestImpl {
    async fn get_app_state(&self) -> Result<HashMap<AppID, AppState>, PortalError>;

    async fn notify_background(&self, app_id: AppID, name: &str)
        -> Result<Background, PortalError>;

    async fn enable_autostart(
        &self,
        app_id: AppID,
        enable: bool,
        commandline: Vec<String>,
        flags: BitFlags<AutoStartFlags>,
    ) -> Result<bool, PortalError>;
}

pub struct BackgroundInterface {
    imp: Arc<dyn BackgroundImpl>,
    cnx: zbus::Connection,
}

impl BackgroundInterface {
    pub fn new(imp: impl BackgroundImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
            cnx,
        }
    }

    pub async fn changed(&self) -> zbus::Result<()> {
        let object_server = self.cnx.object_server();
        let iface_ref = object_server
            .interface::<_, Self>(crate::proxy::DESKTOP_PATH)
            .await?;
        Self::running_applications_changed(iface_ref.signal_context()).await
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Background")]
impl BackgroundInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        2
    }

    #[dbus_interface(out_args("apps"))]
    async fn get_app_state(&self) -> Result<HashMap<AppID, AppState>, PortalError> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Background::GetAppState");

        let response = self.imp.get_app_state().await;

        #[cfg(feature = "tracing")]
        tracing::debug!("Background::GetAppState returned {:#?}", response);
        response
    }

    #[dbus_interface(out_args("response", "results"))]
    async fn notify_background(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        name: String,
    ) -> Result<Response<Background>, PortalError> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Background::NotifyBackground",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move { imp.notify_background(app_id, &name).await },
        )
        .await
    }

    #[dbus_interface(out_args("result"))]
    async fn enable_autostart(
        &self,
        app_id: AppID,
        enable: bool,
        commandline: Vec<String>,
        flags: BitFlags<AutoStartFlags>,
    ) -> Result<bool, PortalError> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Background::EnableAutostart");

        let response = self
            .imp
            .enable_autostart(app_id, enable, commandline, flags)
            .await;

        #[cfg(feature = "tracing")]
        tracing::debug!("Background::EnableAutostart returned {:#?}", response);
        response
    }

    #[zbus(signal)]
    async fn running_applications_changed(signal_ctxt: &SignalContext<'_>) -> zbus::Result<()>;
}
