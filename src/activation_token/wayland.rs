use wayland_backend::sys::client::Backend;
use wayland_client::{
    protocol::{wl_registry, wl_surface::WlSurface},
    Proxy, QueueHandle,
};
use wayland_protocols::xdg::activation::v1::client::{
    xdg_activation_token_v1::{Event, XdgActivationTokenV1},
    xdg_activation_v1::XdgActivationV1,
};

// Supported versions.
const XDG_ACTIVATION_V1_VERSION: u32 = 1;

#[derive(Debug, Default)]
pub struct WaylandActivationToken {
    pub(crate) token: String,
}

#[derive(Default)]
struct WaylandHandle {
    wl_token: Option<XdgActivationTokenV1>,
    raw_token: Option<String>,
}

impl Drop for WaylandHandle {
    fn drop(&mut self) {
        if let Some(wl_token) = self.wl_token.take() {
            wl_token.destroy();
        }
    }
}

impl WaylandActivationToken {
    // Can be changed to display.
    pub async fn from_surface(app_id: &str, surface: &WlSurface) -> Option<Self> {
        let backend = surface.backend().upgrade()?;
        let conn = wayland_client::Connection::from_backend(backend);

        Self::new_inner(app_id, conn, surface).await
    }

    pub async unsafe fn from_raw(
        app_id: &str,
        surface_ptr: *mut std::ffi::c_void,
        display_ptr: *mut std::ffi::c_void,
    ) -> Option<Self> {
        if surface_ptr.is_null() || display_ptr.is_null() {
            return None;
        }

        let backend = Backend::from_foreign_display(display_ptr as *mut _);
        let conn = wayland_client::Connection::from_backend(backend);
        let obj_id = wayland_backend::sys::client::ObjectId::from_ptr(
            WlSurface::interface(),
            surface_ptr as *mut _,
        )
        .ok()?;

        let surface = WlSurface::from_id(&conn, obj_id).ok()?;

        Self::new_inner(app_id, conn, &surface).await
    }

    async fn new_inner(
        app_id: &str,
        conn: wayland_client::Connection,
        surface: &WlSurface,
    ) -> Option<Self> {
        let (sender, receiver) = futures_channel::oneshot::channel::<Option<Self>>();

        // Cheap clone, protocol objects are essentially smart pointers
        let surface = surface.clone();
        let app_id = app_id.to_owned();
        std::thread::spawn(move || match wayland_export_token(app_id, conn, &surface) {
            Ok(window_handle) => sender.send(Some(window_handle)).unwrap(),
            Err(_err) => {
                #[cfg(feature = "tracing")]
                tracing::info!("Could not get wayland window identifier: {_err}");
                sender.send(None).unwrap();
            }
        });

        receiver.await.unwrap()
    }
}

impl wayland_client::Dispatch<XdgActivationTokenV1, ()> for WaylandHandle {
    fn event(
        state: &mut Self,
        _proxy: &XdgActivationTokenV1,
        event: <XdgActivationTokenV1 as Proxy>::Event,
        _data: &(),
        _connhandle: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let Event::Done { token } = event {
            state.raw_token = Some(token);
        }
    }
}

impl wayland_client::Dispatch<wl_registry::WlRegistry, ()> for WaylandHandle {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _connhandle: &wayland_client::Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if &interface == "xdg_activation_v1" {
                #[cfg(feature = "tracing")]
                tracing::info!("Found wayland interface {interface} v{version}");
                let wl_activation = registry.bind::<XdgActivationV1, (), Self>(
                    name,
                    version.min(XDG_ACTIVATION_V1_VERSION),
                    qhandle,
                    (),
                );
                let wl_token = wl_activation.get_activation_token(qhandle, ());
                state.wl_token = Some(wl_token);
                wl_activation.destroy();
            }
        }
    }
}

impl wayland_client::Dispatch<XdgActivationV1, ()> for WaylandHandle {
    fn event(
        _state: &mut Self,
        _activation: &XdgActivationV1,
        _event: wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::Event,
        _data: &(),
        _connhandle: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

fn wayland_export_token(
    app_id: String,
    conn: wayland_client::Connection,
    surface: &WlSurface,
) -> Result<WaylandActivationToken, Box<dyn std::error::Error>> {
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let mut state = WaylandHandle::default();
    let qhandle = event_queue.handle();
    display.get_registry(&qhandle, ());
    event_queue.roundtrip(&mut state)?;

    if let Some(wl_token) = state.wl_token.take() {
        // TODO is this an APP ID in the traditional sense?
        wl_token.set_app_id(app_id);
        wl_token.set_surface(surface);
        // TODO wl_token.set_serial(serial, &seat);
        wl_token.commit();

        event_queue.roundtrip(&mut state)?;
    };

    if let Some(raw_token) = state.raw_token.take() {
        Ok(WaylandActivationToken { token: raw_token })
    } else {
        #[cfg(feature = "tracing")]
        tracing::error!("Failed to get a response from the wayland server");

        Err(Box::new(crate::Error::NoResponse))
    }
}
