use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
    sync::Mutex,
};

use futures_util::StreamExt;
use serde::{
    de::{self, Error as SeError, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize,
};
use zbus::{
    zvariant::{ObjectPath, Signature, Type, Value},
    SignalStream,
};

use crate::{desktop::HandleToken, helpers::session_connection, proxy::Proxy, Error};

/// A typical response returned by the [`Request::receive_response`] signal
/// of a [`Request`].
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
        <(ResponseType, HashMap<&str, Value<'_>>)>::signature()
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

impl<T> Serialize for Response<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Type + Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_tuple(2)?;
        match self {
            Self::Err(err) => {
                map.serialize_element(&ResponseType::from(*err))?;
                map.serialize_element(&T::default())?;
            }
            Self::Ok(response) => {
                map.serialize_element(&ResponseType::Success)?;
                map.serialize_element(response)?;
            }
        };
        map.end()
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

#[derive(Debug, Copy, PartialEq, Eq, Hash, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Type)]
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
/// [`close()`][`Request::close`] on the Request object.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Request`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Request).
#[doc(alias = "org.freedesktop.portal.Request")]
pub struct Request<'a, T>(
    Proxy<'a>,
    SignalStream<'a>,
    Mutex<Option<Result<T, Error>>>,
    PhantomData<T>,
)
where
    T: for<'de> Deserialize<'de> + Type + Debug;

impl<'a, T> Request<'a, T>
where
    T: for<'de> Deserialize<'de> + Type + Debug,
{
    pub async fn new<P>(path: P) -> Result<Request<'a, T>, Error>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let proxy = Proxy::new_desktop_with_path("org.freedesktop.portal.Request", path).await?;
        // Start listening for a response signal the moment request is created
        let stream = proxy.receive_signal("Response").await?;
        Ok(Self(proxy, stream, Default::default(), PhantomData))
    }

    pub async fn from_unique_name(handle_token: &HandleToken) -> Result<Request<'a, T>, Error> {
        let connection = session_connection().await?;
        let unique_name = connection.unique_name().unwrap();
        let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
        let path = ObjectPath::try_from(format!(
            "/org/freedesktop/portal/desktop/request/{unique_identifier}/{handle_token}",
        ))
        .unwrap();
        #[cfg(feature = "tracing")]
        tracing::info!("Creating a org.freedesktop.portal.Request {}", path);
        Self::new(path).await
    }

    pub(crate) async fn prepare_response(&mut self) -> Result<(), Error> {
        let message = self.1.next().await.ok_or(Error::NoResponse)?;
        #[cfg(feature = "tracing")]
        tracing::info!("Received signal 'Response' on '{}'", self.0.interface());
        let response = match message.body::<Response<T>>()? {
            Response::Err(e) => Err(e.into()),
            Response::Ok(r) => Ok(r),
        };
        #[cfg(feature = "tracing")]
        tracing::debug!("Received response {:#?}", response);
        let r = response as Result<T, Error>;
        *self.2.get_mut().unwrap() = Some(r);
        Ok(())
    }

    pub fn response(&self) -> Result<T, Error> {
        // It should be safe to unwrap here as we are sure we have received a response
        // by the time the user calls response
        self.2.lock().unwrap().take().unwrap()
    }

    /// Closes the portal request to which this object refers and ends all
    /// related user interaction (dialogs, etc). A Response signal will not
    /// be emitted in this case.
    ///
    /// # Specifications
    ///
    /// See also [`Close`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Request.Close).
    #[doc(alias = "Close")]
    pub async fn close(&self) -> Result<(), Error> {
        self.0.call_method("Close", &()).await
    }

    pub(crate) fn path(&self) -> &ObjectPath<'_> {
        self.0.path()
    }
}

impl<'a, T> Debug for Request<'a, T>
where
    T: for<'de> Deserialize<'de> + Type + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Request")
            .field(&self.path().as_str())
            .finish()
    }
}
