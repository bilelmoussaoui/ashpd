use std::{collections::HashMap, fmt, marker::PhantomData};

use serde::{
    de::{self, DeserializeOwned, Error, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::dbus_proxy;
use zvariant::OwnedValue;
use zvariant_derive::Type;

/// A typical response returned by the `connect_response` signal of a
/// `RequestProxy`.
///
/// [`RequestProxy`]: ./struct.RequestProxy.html
#[derive(Debug)]
pub enum Response<T>
where
    T: DeserializeOwned + zvariant::Type,
{
    /// Success, the request is carried out.
    Ok(T),
    /// The user cancelled the request or something else happened.
    Err(ResponseError),
}

impl<T> Response<T>
where
    T: DeserializeOwned + zvariant::Type,
{
    /// Whether the request was successful.
    pub fn is_ok(&self) -> bool {
        matches!(self, Response::Ok(_))
    }

    /// Whether the request failed.
    pub fn is_err(&self) -> bool {
        matches!(self, Response::Err(_))
    }

    /// Unwrap the inner response if the request was successful
    ///
    /// # Panic
    ///
    /// The function panics if the request failed and there's no valid response.
    pub fn unwrap(&self) -> &T {
        match self {
            Self::Ok(response) => response,
            Self::Err(_) => panic!("Called Response::unwrap on a Response::Err"),
        }
    }
}

impl<T> zvariant::Type for Response<T>
where
    T: DeserializeOwned + zvariant::Type,
{
    fn signature() -> zvariant::Signature<'static> {
        <(ResponseType, OwnedValue)>::signature()
    }
}

impl<'de, T> Deserialize<'de> for Response<T>
where
    T: DeserializeOwned + zvariant::Type,
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
            type Value = (ResponseType, T);

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
                let data: T = seq.next_element()?.ok_or_else(|| A::Error::custom(
                    "Failed to deserialize the response. Expected a vardict (a{sv}) with the returned results",
                ))?;
                Ok((type_, data))
            }
        }

        let visitor = ResponseVisitor::<T>(PhantomData);
        let response: (ResponseType, T) = deserializer.deserialize_tuple(2, visitor)?;
        Ok(response.into())
    }
}

#[doc(hidden)]
impl<T> From<(ResponseType, T)> for Response<T>
where
    T: DeserializeOwned + zvariant::Type,
{
    fn from(f: (ResponseType, T)) -> Self {
        match f.0 {
            ResponseType::Success => Response::Ok(f.1),
            ResponseType::Cancelled => Response::Err(ResponseError::Cancelled),
            ResponseType::Other => Response::Err(ResponseError::Other),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
/// The most basic response. Used when only the status of the request is what we
/// receive as a response.
pub struct BasicResponse(HashMap<String, OwnedValue>);

impl Default for BasicResponse {
    fn default() -> Self {
        BasicResponse(HashMap::new())
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

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
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
/// The application can abort the interaction calling Close() on the Request
/// object.
///
/// Since version 0.9 of xdg-desktop-portal, the handle will be of the form
/// `/org/freedesktop/portal/desktop/request/SENDER/TOKEN`, where SENDER is the
/// callers unique name,  with the initial ':' removed and all '.' replaced by
/// '_', and TOKEN is a unique token that the caller provided with the
/// handle_token key in the options vardict.
///
/// This change was made to let applications subscribe to the Response signal
/// before making the initial portal call, thereby avoiding a race condition.
/// It is recommended that the caller should verify that the returned handle is
/// what it expected, and update its signal subscription if it isn't.
/// This ensures that applications will work with both old and new versions of
/// xdg-desktop-portal.

#[dbus_proxy(
    default_service = "org.freedesktop.portal.Desktop",
    interface = "org.freedesktop.portal.Request",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Request {
    #[dbus_proxy(signal)]
    /// A signal emitted when the portal interaction is over.
    fn response<T>(&self, response: Response<T>) -> Result<()>
    where
        T: DeserializeOwned + zvariant::Type;

    /// Closes the portal request to which this object refers and ends all
    /// related user interaction (dialogs, etc). A Response signal will not
    /// be emitted in this case.
    fn close(&self) -> zbus::Result<()>;
}
