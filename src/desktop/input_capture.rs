//! # Examples
//!
//! ## A Note of Warning Regarding the GNOME Portal Implementation
//!
//! `xdg-desktop-portal-gnome` in version 46.0 has a
//! [bug](https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/issues/126)
//! that prevents reenabling a disabled session.
//!
//! Since changing barrier locations requires a session to be disabled,
//! it is currently (as of GNOME 46) not possible to change barriers
//! after a session has been enabled!
//!
//! (the [official documentation](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-setpointerbarriers)
//! states that a
//! [`InputCapture::set_pointer_barriers()`][set_pointer_barriers]
//! request suspends the capture session but in reality the GNOME
//! desktop portal enforces a
//! [`InputCapture::disable()`][disable]
//! request
//! in order to use
//! [`InputCapture::set_pointer_barriers()`][set_pointer_barriers]
//! )
//!
//! [set_pointer_barriers]: crate::desktop::input_capture::InputCapture::set_pointer_barriers
//! [disable]: crate::desktop::input_capture::InputCapture::disable
//!
//! ## Retrieving an Ei File Descriptor
//!
//! The input capture portal is used to negotiate the input capture
//! triggers and enable input capturing.
//!
//! Actual input capture events are then communicated over a unix
//! stream using the [libei protocol](https://gitlab.freedesktop.org/libinput/libei).
//!
//! The lifetime of an ei file descriptor is bound by a capture session.
//!
//! ```rust,no_run
//! use std::os::fd::AsRawFd;
//!
//! use ashpd::desktop::input_capture::{Capabilities, InputCapture};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let input_capture = InputCapture::new().await?;
//!     let (session, capabilities) = input_capture
//!         .create_session(
//!             &ashpd::WindowIdentifier::default(),
//!             Capabilities::Keyboard | Capabilities::Pointer | Capabilities::Touchscreen,
//!         )
//!         .await?;
//!     eprintln!("capabilities: {capabilities}");
//!
//!     let eifd = input_capture.connect_to_eis(&session).await?;
//!     eprintln!("eifd: {}", eifd.as_raw_fd());
//!     Ok(())
//! }
//! ```
//!
//!
//! ## Selecting Pointer Barriers.
//!
//! Input capture is triggered through pointer barriers that are provided
//! by the client.
//!
//! The provided barriers need to be positioned at the edges of outputs
//! (monitors) and can be denied by the compositor for various reasons, such as
//! wrong placement.
//!
//! For debugging why a barrier placement failed, the logs of the
//! active portal implementation can be useful, e.g.:
//!
//! ```sh
//! journalctl --user -xeu xdg-desktop-portal-gnome.service
//! ```
//!
//! The following example sets up barriers according to `pos`
//! (either `Left`, `Right`, `Top` or `Bottom`).
//!
//! Note that barriers positioned between two monitors will be denied
//! and returned in the `failed_barrier_ids` vector.
//!
//! ```rust,no_run
//! use ashpd::desktop::input_capture::{Barrier, Capabilities, InputCapture};
//!
//! #[allow(unused)]
//! enum Position {
//!     Left,
//!     Right,
//!     Top,
//!     Bottom,
//! }
//!
//! async fn run() -> ashpd::Result<()> {
//!     let input_capture = InputCapture::new().await?;
//!     let (session, _capabilities) = input_capture
//!         .create_session(
//!             &ashpd::WindowIdentifier::default(),
//!             Capabilities::Keyboard | Capabilities::Pointer | Capabilities::Touchscreen,
//!         )
//!         .await?;
//!
//!     let pos = Position::Left;
//!     let zones = input_capture.zones(&session).await?.response()?;
//!     eprintln!("zones: {zones:?}");
//!     let barriers = zones
//!         .regions()
//!         .iter()
//!         .enumerate()
//!         .map(|(n, r)| {
//!             let id = n as u32;
//!             let (x, y) = (r.x_offset(), r.y_offset());
//!             let (width, height) = (r.width() as i32, r.height() as i32);
//!             let barrier_pos = match pos {
//!                 Position::Left => (x, y, x, y + height - 1), // start pos, end pos, inclusive
//!                 Position::Right => (x + width, y, x + width, y + height - 1),
//!                 Position::Top => (x, y, x + width - 1, y),
//!                 Position::Bottom => (x, y + height, x + width - 1, y + height),
//!             };
//!             Barrier::new(id, barrier_pos)
//!         })
//!         .collect::<Vec<_>>();
//!
//!     eprintln!("requested barriers: {barriers:?}");
//!
//!     let request = input_capture
//!         .set_pointer_barriers(&session, &barriers, zones.zone_set())
//!         .await?;
//!     let response = request.response()?;
//!     let failed_barrier_ids = response.failed_barriers();
//!
//!     eprintln!("failed barrier ids: {:?}", failed_barrier_ids);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Enabling Input Capture and Retrieving Captured Input Events.
//!
//! The following full example uses the [reis crate](https://docs.rs/reis/0.2.0/reis/)
//! for libei communication.
//!
//! Input Capture can be released using ESC.
//!
//! ```rust,no_run
//! use std::{collections::HashMap, os::unix::net::UnixStream, sync::OnceLock, time::Duration};
//!
//! use ashpd::desktop::input_capture::{Barrier, Capabilities, InputCapture};
//! use futures_util::StreamExt;
//! use reis::{
//!     ei::{self, keyboard::KeyState},
//!     event::{DeviceCapability, EiEvent, KeyboardKey},
//!     tokio::{EiConvertEventStream, EiEventStream},
//! };
//!
//! #[allow(unused)]
//! enum Position {
//!     Left,
//!     Right,
//!     Top,
//!     Bottom,
//! }
//!
//! static INTERFACES: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
//!
//! async fn run() -> ashpd::Result<()> {
//!     let input_capture = InputCapture::new().await?;
//!
//!     let (session, _cap) = input_capture
//!         .create_session(
//!             &ashpd::WindowIdentifier::default(),
//!             Capabilities::Keyboard | Capabilities::Pointer | Capabilities::Touchscreen,
//!         )
//!         .await?;
//!
//!     // connect to eis server
//!     let fd = input_capture.connect_to_eis(&session).await?;
//!
//!     // create unix stream from fd
//!     let stream = UnixStream::from(fd);
//!     stream.set_nonblocking(true)?;
//!
//!     // create ei context
//!     let context = ei::Context::new(stream)?;
//!     context.flush().unwrap();
//!
//!     let mut event_stream = EiEventStream::new(context.clone())?;
//!     let interfaces = INTERFACES.get_or_init(|| {
//!         HashMap::from([
//!             ("ei_connection", 1),
//!             ("ei_callback", 1),
//!             ("ei_pingpong", 1),
//!             ("ei_seat", 1),
//!             ("ei_device", 2),
//!             ("ei_pointer", 1),
//!             ("ei_pointer_absolute", 1),
//!             ("ei_scroll", 1),
//!             ("ei_button", 1),
//!             ("ei_keyboard", 1),
//!             ("ei_touchscreen", 1),
//!         ])
//!     });
//!     let response = reis::tokio::ei_handshake(
//!         &mut event_stream,
//!         "ashpd-mre",
//!         ei::handshake::ContextType::Receiver,
//!         interfaces,
//!     )
//!     .await
//!     .expect("ei handshake failed");
//!
//!     let mut event_stream = EiConvertEventStream::new(event_stream, response.serial);
//!
//!     let pos = Position::Left;
//!     let zones = input_capture.zones(&session).await?.response()?;
//!     eprintln!("zones: {zones:?}");
//!     let barriers = zones
//!         .regions()
//!         .iter()
//!         .enumerate()
//!         .map(|(n, r)| {
//!             let id = n as u32;
//!             let (x, y) = (r.x_offset(), r.y_offset());
//!             let (width, height) = (r.width() as i32, r.height() as i32);
//!             let barrier_pos = match pos {
//!                 Position::Left => (x, y, x, y + height - 1), // start pos, end pos, inclusive
//!                 Position::Right => (x + width, y, x + width, y + height - 1),
//!                 Position::Top => (x, y, x + width - 1, y),
//!                 Position::Bottom => (x, y + height, x + width - 1, y + height),
//!             };
//!             Barrier::new(id, barrier_pos)
//!         })
//!         .collect::<Vec<_>>();
//!
//!     eprintln!("requested barriers: {barriers:?}");
//!
//!     let request = input_capture
//!         .set_pointer_barriers(&session, &barriers, zones.zone_set())
//!         .await?;
//!     let response = request.response()?;
//!     let failed_barrier_ids = response.failed_barriers();
//!
//!     eprintln!("failed barrier ids: {:?}", failed_barrier_ids);
//!
//!     input_capture.enable(&session).await?;
//!
//!     let mut activate_stream = input_capture.receive_activated().await?;
//!
//!     loop {
//!         let activated = activate_stream.next().await.unwrap();
//!
//!         eprintln!("activated: {activated:?}");
//!         loop {
//!             let ei_event = event_stream.next().await.unwrap().unwrap();
//!             eprintln!("ei event: {ei_event:?}");
//!             if let EiEvent::SeatAdded(seat_event) = &ei_event {
//!                 seat_event.seat.bind_capabilities(&[
//!                     DeviceCapability::Pointer,
//!                     DeviceCapability::PointerAbsolute,
//!                     DeviceCapability::Keyboard,
//!                     DeviceCapability::Touch,
//!                     DeviceCapability::Scroll,
//!                     DeviceCapability::Button,
//!                 ]);
//!                 context.flush().unwrap();
//!             }
//!             if let EiEvent::DeviceAdded(_) = ei_event {
//!                 // new device added -> restart capture
//!                 break;
//!             };
//!             if let EiEvent::KeyboardKey(KeyboardKey { key, state, .. }) = ei_event {
//!                 if key == 1 && state == KeyState::Press {
//!                     // esc pressed
//!                     break;
//!                 }
//!             }
//!         }
//!
//!         eprintln!("releasing input capture");
//!         let (x, y) = activated.cursor_position();
//!         let (x, y) = (x as f64, y as f64);
//!         let cursor_pos = match pos {
//!             Position::Left => (x + 1., y),
//!             Position::Right => (x - 1., y),
//!             Position::Top => (x, y - 1.),
//!             Position::Bottom => (x, y + 1.),
//!         };
//!         input_capture
//!             .release(&session, activated.activation_id(), cursor_pos)
//!             .await?;
//!     }
//! }
//! ```

use std::{collections::HashMap, os::fd::OwnedFd};

use enumflags2::{bitflags, BitFlags};
use futures_util::{Stream, TryFutureExt};
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{
    self, DeserializeDict, ObjectPath, OwnedObjectPath, OwnedValue, SerializeDict, Type, Value,
};

use super::{session::SessionPortal, HandleToken, Request, Session};
use crate::{proxy::Proxy, Error, WindowIdentifier};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Copy, Clone, Type)]
#[bitflags]
#[repr(u32)]
/// Supported capabilities
pub enum Capabilities {
    /// Keyboard
    Keyboard,
    /// Pointer
    Pointer,
    /// Touchscreen
    Touchscreen,
}

#[derive(Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct CreateSessionOptions {
    handle_token: HandleToken,
    session_handle_token: HandleToken,
    capabilities: BitFlags<Capabilities>,
}

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
struct CreateSessionResponse {
    session_handle: OwnedObjectPath,
    capabilities: BitFlags<Capabilities>,
}

#[derive(Default, Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct GetZonesOptions {
    handle_token: HandleToken,
}

#[derive(Default, Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct SetPointerBarriersOptions {
    handle_token: HandleToken,
}

#[derive(Default, Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct EnableOptions {}

#[derive(Default, Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct DisableOptions {}

#[derive(Default, Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct ReleaseOptions {
    activation_id: u32,
    cursor_position: (f64, f64),
}

/// Indicates that an input capturing session was disabled.
#[derive(Debug, Deserialize, Type)]
#[zvariant(signature = "(oa{sv})")]
pub struct Disabled(OwnedObjectPath, HashMap<String, OwnedValue>);

impl Disabled {
    /// Session that was disabled.
    pub fn session_handle(&self) -> ObjectPath<'_> {
        self.0.as_ref()
    }

    /// Optional information
    pub fn options(&self) -> &HashMap<String, OwnedValue> {
        &self.1
    }
}

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
struct DeactivatedOptions {
    activation_id: u32,
}

/// Indicates that an input capturing session was deactivated.
#[derive(Debug, Deserialize, Type)]
#[zvariant(signature = "(oa{sv})")]
pub struct Deactivated(OwnedObjectPath, DeactivatedOptions);

impl Deactivated {
    /// Session that was deactivated.
    pub fn session_handle(&self) -> ObjectPath<'_> {
        self.0.as_ref()
    }

    /// The same activation_id number as in the corresponding "Activated"
    /// signal.
    pub fn activation_id(&self) -> u32 {
        self.1.activation_id
    }
}

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
struct ActivatedOptions {
    activation_id: u32,
    cursor_position: (f32, f32),
    barrier_id: BarrierID,
}

/// Indicates that an input capturing session was activated.
#[derive(Debug, Deserialize, Type)]
#[zvariant(signature = "(oa{sv})")]
pub struct Activated(OwnedObjectPath, ActivatedOptions);

impl Activated {
    /// Session that was activated.
    pub fn session_handle(&self) -> ObjectPath<'_> {
        self.0.as_ref()
    }

    /// A number that can be used to synchronize with the transport-layer.
    pub fn activation_id(&self) -> u32 {
        self.1.activation_id
    }

    /// The current cursor position in the same coordinate space as the zones.
    pub fn cursor_position(&self) -> (f32, f32) {
        self.1.cursor_position
    }

    /// The barrier id of the barrier that triggered
    pub fn barrier_id(&self) -> BarrierID {
        self.1.barrier_id
    }
}

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
struct ZonesChangedOptions {
    zone_set: u32,
}

/// Indicates that zones available to this session changed.
#[derive(Debug, Deserialize, Type)]
#[zvariant(signature = "(oa{sv})")]
pub struct ZonesChanged(OwnedObjectPath, ZonesChangedOptions);

impl ZonesChanged {
    /// Session that was deactivated.
    pub fn session_handle(&self) -> ObjectPath<'_> {
        self.0.as_ref()
    }

    ///  The zone_set ID of the invalidated zone.
    pub fn zone_set(&self) -> u32 {
        self.1.zone_set
    }
}

/// A region of a [`Zones`].
#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[zvariant(signature = "(uuii)")]
pub struct Region(u32, u32, i32, i32);

impl Region {
    /// The width.
    pub fn width(self) -> u32 {
        self.0
    }

    /// The height
    pub fn height(self) -> u32 {
        self.1
    }

    /// The x offset.
    pub fn x_offset(self) -> i32 {
        self.2
    }

    /// The y offset.
    pub fn y_offset(self) -> i32 {
        self.3
    }
}

/// A response of [`InputCapture::zones`].
#[derive(Debug, Type, DeserializeDict)]
#[zvariant(signature = "dict")]
pub struct Zones {
    zones: Vec<Region>,
    zone_set: u32,
}

impl Zones {
    /// A list of regions.
    pub fn regions(&self) -> &[Region] {
        &self.zones
    }

    /// A unique ID to be used in [`InputCapture::set_pointer_barriers`].
    pub fn zone_set(&self) -> u32 {
        self.zone_set
    }
}

/// A barrier ID.
pub type BarrierID = u32;

#[derive(Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
/// Input Barrier.
pub struct Barrier {
    barrier_id: BarrierID,
    position: (i32, i32, i32, i32),
}

impl Barrier {
    /// Create a new barrier.
    pub fn new(barrier_id: BarrierID, position: (i32, i32, i32, i32)) -> Self {
        Self {
            barrier_id,
            position,
        }
    }
}

/// A response to [`InputCapture::set_pointer_barriers`]
#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct SetPointerBarriersResponse {
    failed_barriers: Vec<BarrierID>,
}

impl SetPointerBarriersResponse {
    /// List of pointer barriers that have been denied
    pub fn failed_barriers(&self) -> &[BarrierID] {
        &self.failed_barriers
    }
}

/// Wrapper of the DBus interface: [`org.freedesktop.portal.InputCapture`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html).
#[doc(alias = "org.freedesktop.portal.InputCapture")]
pub struct InputCapture<'a>(Proxy<'a>);

impl<'a> InputCapture<'a> {
    /// Create a new instance of [`InputCapture`].
    pub async fn new() -> Result<InputCapture<'a>, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.InputCapture").await?;
        Ok(Self(proxy))
    }

    /// Create an input capture session.
    ///
    /// # Specifications
    ///
    /// See also [`CreateSession`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-createsession).
    pub async fn create_session(
        &self,
        parent_window: &WindowIdentifier,
        capabilities: BitFlags<Capabilities>,
    ) -> Result<(Session<'_, Self>, BitFlags<Capabilities>), Error> {
        let options = CreateSessionOptions {
            handle_token: Default::default(),
            session_handle_token: Default::default(),
            capabilities,
        };

        let (request, proxy) = futures_util::try_join!(
            self.0
                .request::<CreateSessionResponse>(
                    &options.handle_token,
                    "CreateSession",
                    (parent_window, &options)
                )
                .into_future(),
            Session::from_unique_name(&options.session_handle_token).into_future(),
        )?;
        let response = request.response()?;
        assert_eq!(proxy.path(), &response.session_handle.as_ref());
        Ok((proxy, response.capabilities))
    }

    /// A set of currently available input zones for this session.
    ///
    /// # Specifications
    ///
    /// See also [`GetZones`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-getzones).
    #[doc(alias = "GetZones")]
    pub async fn zones(&self, session: &Session<'_, Self>) -> Result<Request<Zones>, Error> {
        let options = GetZonesOptions::default();
        self.0
            .request(&options.handle_token, "GetZones", (session, &options))
            .await
    }

    /// Set up zero or more pointer barriers.
    ///
    /// # Specifications
    ///
    /// See also [`SetPointerBarriers`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-setpointerbarriers).
    #[doc(alias = "SetPointerBarriers")]
    pub async fn set_pointer_barriers(
        &self,
        session: &Session<'_, Self>,
        barriers: &[Barrier],
        zone_set: u32,
    ) -> Result<Request<SetPointerBarriersResponse>, Error> {
        let options = SetPointerBarriersOptions::default();
        self.0
            .request(
                &options.handle_token,
                "SetPointerBarriers",
                &(session, &options, barriers, zone_set),
            )
            .await
    }

    /// Enable input capturing.
    ///
    /// # Specifications
    ///
    /// See also [`Enable`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-enable).
    pub async fn enable(&self, session: &Session<'_, Self>) -> Result<(), Error> {
        let options = EnableOptions::default();
        self.0.call("Enable", &(session, &options)).await
    }

    /// Disable input capturing.
    ///
    /// # Specifications
    ///
    /// See also [`Disable`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-disable).
    pub async fn disable(&self, session: &Session<'_, Self>) -> Result<(), Error> {
        let options = DisableOptions::default();
        self.0.call("Disable", &(session, &options)).await
    }

    /// Release any ongoing input capture.
    ///
    /// # Specifications
    ///
    /// See also [`Release`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-release).
    pub async fn release(
        &self,
        session: &Session<'_, Self>,
        activation_id: u32,
        cursor_position: (f64, f64),
    ) -> Result<(), Error> {
        let options = ReleaseOptions {
            activation_id,
            cursor_position,
        };
        self.0.call("Release", &(session, &options)).await
    }

    /// Connect to EIS.
    ///
    /// # Specifications
    ///
    /// See also [`ConnectToEIS`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-connecttoeis).
    #[doc(alias = "ConnectToEIS")]
    pub async fn connect_to_eis(&self, session: &Session<'_, Self>) -> Result<OwnedFd, Error> {
        // `ConnectToEIS` doesn't take any options for now
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let fd = self
            .0
            .call::<zvariant::OwnedFd>("ConnectToEIS", &(session, options))
            .await?;
        Ok(fd.into())
    }

    /// Signal emitted when the application will no longer receive captured
    /// events.
    ///
    /// # Specifications
    ///
    /// See also [`Disabled`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-disabled).
    #[doc(alias = "Disabled")]
    pub async fn receive_disabled(&self) -> Result<impl Stream<Item = Disabled>, Error> {
        self.0.signal("Disabled").await
    }

    /// Signal emitted when input capture starts and
    /// input events are about to be sent to the application.
    ///
    /// # Specifications
    ///
    /// See also [`Activated`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-activated).
    #[doc(alias = "Activated")]
    pub async fn receive_activated(&self) -> Result<impl Stream<Item = Activated>, Error> {
        self.0.signal("Activated").await
    }

    /// Signal emitted when input capture stopped and input events
    /// are no longer sent to the application.
    ///
    /// # Specifications
    ///
    /// See also [`Deactivated`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-deactivated).
    #[doc(alias = "Deactivated")]
    pub async fn receive_deactivated(&self) -> Result<impl Stream<Item = Deactivated>, Error> {
        self.0.signal("Deactivated").await
    }

    /// Signal emitted when the set of zones available to this session change.
    ///
    /// # Specifications
    ///
    /// See also [`ZonesChanged`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-zoneschanged).
    #[doc(alias = "ZonesChanged")]
    pub async fn receive_zones_changed(&self) -> Result<impl Stream<Item = ZonesChanged>, Error> {
        self.0.signal("ZonesChanged").await
    }

    /// Supported capabilities.
    ///
    /// # Specifications
    ///
    /// See also [`SupportedCapabilities`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html#org-freedesktop-portal-inputcapture-supportedcapabilities).
    #[doc(alias = "SupportedCapabilities")]
    pub async fn supported_capabilities(&self) -> Result<BitFlags<Capabilities>, Error> {
        self.0.property("SupportedCapabilities").await
    }
}

impl<'a> std::ops::Deref for InputCapture<'a> {
    type Target = zbus::Proxy<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SessionPortal for InputCapture<'_> {}
