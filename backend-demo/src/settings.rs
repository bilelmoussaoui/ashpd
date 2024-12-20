use std::{collections::HashMap, sync::Arc};

use ashpd::{
    backend::{
        request::RequestImpl,
        settings::{SettingsImpl, SettingsSignalEmitter},
    },
    desktop::{
        settings::{ColorScheme, Namespace, APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY},
        HandleToken,
    },
    zbus::zvariant::OwnedValue,
    PortalError,
};
use async_trait::async_trait;

#[derive(Default, Clone)]
pub struct Settings {
    color_scheme: ColorScheme,
    // The signal emitter allows dispatching changed events without exposing DBus internals
    signal_emitter: Option<Arc<dyn SettingsSignalEmitter>>,
}

#[async_trait]
impl RequestImpl for Settings {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

#[async_trait]
impl SettingsImpl for Settings {
    async fn read_all(
        &self,
        _namespaces: Vec<String>,
    ) -> Result<HashMap<String, Namespace>, PortalError> {
        let mut namespace_map = HashMap::new();
        namespace_map.insert(
            COLOR_SCHEME_KEY.to_owned(),
            OwnedValue::from(self.color_scheme),
        );
        let mut map = HashMap::new();
        map.insert(APPEARANCE_NAMESPACE.to_owned(), namespace_map);
        Ok(map)
    }

    async fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue, PortalError> {
        if namespace == APPEARANCE_NAMESPACE && key == COLOR_SCHEME_KEY {
            Ok(OwnedValue::from(self.color_scheme))
        } else {
            Err(PortalError::Failed(format!(
                "Unsupported namespace=`{namespace}` & key=`{key}`"
            )))
        }
    }

    fn set_signal_emitter(&mut self, signal_emitter: Arc<dyn SettingsSignalEmitter>) {
        self.signal_emitter.replace(signal_emitter);
    }
}
