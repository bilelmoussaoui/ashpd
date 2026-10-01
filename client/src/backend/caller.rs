use zbus::{
    message::Header,
    names::{BusName, OwnedWellKnownName, WellKnownName},
};

use crate::{PortalError, backend::Result};

/// Controls which D-Bus peers may call a portal backend interface.
#[derive(Clone, Debug)]
pub struct CallerAuthorization {
    allowed_names: Option<Vec<OwnedWellKnownName>>,
}

impl CallerAuthorization {
    /// Allow calls from any D-Bus peer.
    pub fn allow_all() -> Self {
        Self {
            allowed_names: None,
        }
    }

    /// Only allow the current owner of `well_known_name`.
    pub fn require_name<'a, W>(well_known_name: W) -> zbus::Result<Self>
    where
        W: TryInto<WellKnownName<'a>>,
        <W as TryInto<WellKnownName<'a>>>::Error: Into<zbus::Error>,
    {
        Self::require_names([well_known_name])
    }

    /// Only allow the current owner of one of `well_known_names`.
    pub fn require_names<'a, I, W>(well_known_names: I) -> zbus::Result<Self>
    where
        I: IntoIterator<Item = W>,
        W: TryInto<WellKnownName<'a>>,
        <W as TryInto<WellKnownName<'a>>>::Error: Into<zbus::Error>,
    {
        let allowed_names = well_known_names
            .into_iter()
            .map(|name| {
                name.try_into()
                    .map(OwnedWellKnownName::from)
                    .map_err(Into::into)
            })
            .collect::<zbus::Result<_>>()?;

        Ok(Self {
            allowed_names: Some(allowed_names),
        })
    }

    pub(crate) async fn authorize(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> Result<()> {
        let Some(allowed_names) = &self.allowed_names else {
            return Ok(());
        };
        let sender = header.sender().ok_or_else(not_allowed)?;
        let proxy = zbus::fdo::DBusProxy::new(connection).await?;

        for name in allowed_names {
            if proxy
                .get_name_owner(BusName::from(name))
                .await
                .is_ok_and(|owner| sender_matches_owner(Some(sender.as_str()), owner.as_str()))
            {
                return Ok(());
            }
        }

        Err(not_allowed())
    }

    pub(crate) async fn authorize_fdo(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> zbus::fdo::Result<()> {
        self.authorize(connection, header)
            .await
            .map_err(|error| zbus::fdo::Error::AccessDenied(error.to_string()))
    }

    pub(crate) async fn authorize_property(
        &self,
        connection: &zbus::Connection,
        header: Option<&Header<'_>>,
    ) -> zbus::fdo::Result<()> {
        let Some(header) = header else {
            // Generated property-changed helpers invoke getters internally.
            return Ok(());
        };

        self.authorize_fdo(connection, header).await
    }
}

fn not_allowed() -> PortalError {
    PortalError::NotAllowed("caller is not authorized to invoke this portal backend".into())
}

fn sender_matches_owner(sender: Option<&str>, owner: &str) -> bool {
    sender == Some(owner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_single_and_multiple_name_policies() {
        let one = CallerAuthorization::require_name("org.freedesktop.portal.Desktop").unwrap();
        assert_eq!(one.allowed_names.unwrap().len(), 1);

        let multiple = CallerAuthorization::require_names([
            "org.freedesktop.portal.Desktop",
            "org.freedesktop.portal.Documents",
        ])
        .unwrap();
        assert_eq!(multiple.allowed_names.unwrap().len(), 2);
    }

    #[test]
    fn rejects_invalid_well_known_names() {
        assert!(CallerAuthorization::require_name(":1.42").is_err());
    }

    #[test]
    fn matches_only_the_current_unique_name() {
        assert!(sender_matches_owner(Some(":1.42"), ":1.42"));
        assert!(!sender_matches_owner(Some(":1.7"), ":1.42"));
        assert!(!sender_matches_owner(None, ":1.42"));
    }
}
