use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
};

use serde::{
    de::{self, Error as SeError, Visitor},
    Deserialize, Deserializer, Serialize,
};
use zbus::zvariant::{ObjectPath, OwnedValue, Signature, Type};

use super::DESTINATION;
use crate::{
    desktop::HandleToken,
    helpers::{call_method, receive_signal},
    Error,
};

/// A typical response returned by the [`RequestProxy::receive_response`] signal
/// of a [`RequestProxy`].
#[derive(Debug)]
pub(crate) enum Response<T>
where
    T: for<'de> Deserialize<'de> + Type,
{
    /// Success, the request is carried out.
    Ok(T),
    /// The user cancelled the request or something else happened.
    Err(ResponseError),
}

impl<T> Type for Response<T>
where
    T: for<'de> Deserialize<'de> + Type,
{
    fn signature() -> Signature<'static> {
        <(ResponseType, HashMap<&str, OwnedValue>)>::signature()
    }
}

impl<'de, T> Deserialize<'de> for Response<T>
where
    T: for<'d> Deserialize<'d> + Type,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResponseVisitor<T>(PhantomData<fn() -> (ResponseType, T)>);

        impl<'de, T> Visitor<'de> for ResponseVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = (ResponseType, Option<T>);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a tuple composed of the response status along with the response"
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let type_: ResponseType = seq.next_element()?.ok_or_else(|| A::Error::custom(
                    "Failed to deserialize the response. Expected a numeric (u) value as the first item of the returned tuple",
                ))?;
                if type_ == ResponseType::Success {
                    let data: T = seq.next_element()?.ok_or_else(|| A::Error::custom(
                        "Failed to deserialize the response. Expected a vardict (a{sv}) with the returned results",
                    ))?;
                    Ok((type_, Some(data)))
                } else {
                    Ok((type_, None))
                }
            }
        }

        let visitor = ResponseVisitor::<T>(PhantomData);
        let response: (ResponseType, Option<T>) = deserializer.deserialize_tuple(2, visitor)?;
        Ok(response.into())
    }
}

#[doc(hidden)]
impl<T> From<(ResponseType, Option<T>)> for Response<T>
where
    T: for<'de> Deserialize<'de> + Type,
{
    fn from(f: (ResponseType, Option<T>)) -> Self {
        match f.0 {
            ResponseType::Success => {
                Response::Ok(f.1.expect("Expected a valid response, found nothing."))
            }
            ResponseType::Cancelled => Response::Err(ResponseError::Cancelled),
            ResponseType::Other => Response::Err(ResponseError::Other),
        }
    }
}

#[derive(Serialize, Deserialize, Type)]
/// The most basic response. Used when only the status of the request is what we
/// receive as a response.
pub(crate) struct BasicResponse(HashMap<String, OwnedValue>);

impl Debug for BasicResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BasicResponse").finish()
    }
}

#[derive(Debug, Copy, PartialEq, Hash, Clone)]
/// An error returned a portal request caused by either the user cancelling the
/// request or something else.
pub enum ResponseError {
    /// The user canceled the request.
    Cancelled,
    /// Something else happened.
    Other,
}

impl std::error::Error for ResponseError {}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("Cancelled"),
            Self::Other => f.write_str("Other"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Type)]
#[doc(hidden)]
enum ResponseType {
    /// Success, the request is carried out.
    Success = 0,
    /// The user cancelled the interaction.
    Cancelled = 1,
    /// The user interaction was ended in some other way.
    Other = 2,
}

#[doc(hidden)]
impl From<ResponseError> for ResponseType {
    fn from(err: ResponseError) -> Self {
        match err {
            ResponseError::Other => Self::Other,
            ResponseError::Cancelled => Self::Cancelled,
        }
    }
}

/// The Request interface is shared by all portal interfaces.
/// When a portal method is called, the reply includes a handle (i.e. object
/// path) for a Request object, which will stay alive for the duration of the
/// user interaction related to the method call.
///
/// The portal indicates that a portal request interaction is over by emitting
/// the "Response" signal on the Request object.
///
/// The application can abort the interaction calling
/// [`close()`][`RequestProxy::close`] on the Request object.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Request`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Request).
#[doc(alias = "org.freedesktop.portal.Request")]
pub(crate) struct RequestProxy<'a>(zbus::Proxy<'a>);

impl<'a> RequestProxy<'a> {
    pub async fn new(
        connection: &zbus::Connection,
        path: ObjectPath<'a>,
    ) -> Result<RequestProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Request")?
            .path(path)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    pub async fn from_unique_name(
        connection: &zbus::Connection,
        handle_token: &HandleToken,
    ) -> Result<RequestProxy<'a>, Error> {
        let unique_name = connection.unique_name().unwrap();
        let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
        let path = ObjectPath::try_from(format!(
            "/org/freedesktop/portal/desktop/request/{}/{}",
            unique_identifier, handle_token
        ))
        .unwrap();
        #[cfg(feature = "log")]
        tracing::info!("Creating a org.freedesktop.portal.Request {}", path);
        RequestProxy::new(connection, path).await
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// See also [`Response`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Request.Response).
    #[doc(alias = "Response")]
    #[allow(dead_code)]
    pub async fn receive_response<R>(&self) -> Result<R, Error>
    where
        R: for<'de> Deserialize<'de> + Type + Debug,
    {
        let response = receive_signal::<Response<R>>(self.inner(), "Response").await?;
        match response {
            Response::Err(e) => Err(e.into()),
            Response::Ok(r) => Ok(r),
        }
    }

    /// Closes the portal request to which this object refers and ends all
    /// related user interaction (dialogs, etc). A Response signal will not
    /// be emitted in this case.
    ///
    /// # Specifications
    ///
    /// See also [`Close`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Request.Close).
    #[allow(dead_code)]
    #[doc(alias = "Close")]
    pub async fn close(&self) -> Result<(), Error> {
        call_method(self.inner(), "Close", &()).await
    }
}

impl<'a> Debug for RequestProxy<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RequestProxy")
            .field(&self.inner().path().as_str())
            .finish()
    }
}
