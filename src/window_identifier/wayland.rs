#[cfg(feature = "wayland")]
use std::fmt;
#[cfg(feature = "wayland")]
use wayland_client::{
    protocol::{__interfaces::WL_SURFACE_INTERFACE, wl_surface::WlSurface},
    ConnectionHandle, Proxy, QueueHandle,
};
#[cfg(feature = "wayland")]
use wayland_protocols::unstable::xdg_foreign::v2::client::{
    zxdg_exported_v2::{Event, ZxdgExportedV2},
    zxdg_exporter_v2::ZxdgExporterV2,
};

#[cfg(feature = "wayland")]
pub struct WaylandWindowIdentifier {
    exported: ZxdgExportedV2,
    handle: String,
}

#[cfg(feature = "wayland")]
impl WaylandWindowIdentifier {
    pub fn new(surface_ptr: *mut std::ffi::c_void) -> Option<Self> {
        match wayland_handle_export(surface_ptr) {
            Ok((exported, handle)) => Some(Self { exported, handle }),
            _ => None,
        }
    }
}

#[cfg(feature = "wayland")]
impl fmt::Display for WaylandWindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&to_handle(&self.handle))
    }
}

#[cfg(feature = "wayland")]
impl Drop for WaylandWindowIdentifier {
    fn drop(&mut self) {
        if let Err(_err) = wayland_handle_unexport(&self.exported) {
            #[cfg(feature = "log")]
            tracing::error!("Failed to unexport wayland handle {}", _err);
        }
    }
}

#[cfg(feature = "wayland")]
#[derive(Default, Debug)]
struct ExportedWaylandHandle(String);

#[cfg(feature = "wayland")]
impl wayland_client::Dispatch<ZxdgExportedV2> for ExportedWaylandHandle {
    type UserData = ();

    fn event(
        &mut self,
        _proxy: &ZxdgExportedV2,
        event: <ZxdgExportedV2 as Proxy>::Event,
        _data: &Self::UserData,
        _connhandle: &mut ConnectionHandle<'_>,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            Event::Handle { handle } => {
                self.0 = handle;
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "wayland")]
/// A helper to export a wayland handle from a WLSurface
///
/// Needed for converting a RawWindowHandle to a WindowIdentifier
fn wayland_handle_export(
    surface_ptr: *mut std::ffi::c_void,
) -> Result<(ZxdgExportedV2, String), Box<dyn std::error::Error>> {
    let cnx = wayland_client::Connection::connect_to_env()?;
    let mut handle = cnx.handle();
    let surface_id = unsafe {
        wayland_backend::sys::client::ObjectId::from_ptr(
            &WL_SURFACE_INTERFACE,
            surface_ptr as *mut _,
        )?
    };
    let surface = WlSurface::from_id(&mut handle, surface_id)?;

    let exporter = ZxdgExporterV2::from_id(&mut handle, surface.id())?;
    let mut queue = cnx.new_event_queue();
    let mut wl_handle = ExportedWaylandHandle::default();

    let queue_handle = queue.handle();
    let exported = exporter.export_toplevel(&mut handle, &surface, &queue_handle, ())?;
    queue.blocking_dispatch(&mut wl_handle)?;
    Ok((exported, wl_handle.0))
}

#[cfg(feature = "wayland")]
/// A helper to unexport a wayland handle from a previously exported one
///
/// Needed for converting a RawWindowHandle to a WindowIdentifier
fn wayland_handle_unexport(exported: &ZxdgExportedV2) -> Result<(), Box<dyn std::error::Error>> {
    let cnx = wayland_client::Connection::connect_to_env()?;
    let mut handle = cnx.handle();
    exported.destroy(&mut handle);

    Ok(())
}

pub fn to_handle(wayland_handle: &str) -> String {
    format!("wayland:{}", wayland_handle)
}
