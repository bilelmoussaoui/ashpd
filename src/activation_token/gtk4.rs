use gtk4::{glib::translate::*, prelude::*};

use crate::ActivationToken;

impl ActivationToken {
    /// Gets an activation token from a window.
    ///
    /// Support for the XDG Activation Protocol was added in GLib 2.76, this
    /// method will return `None` on older versions.
    pub fn from_window(widget: &impl IsA<::gtk4::Widget>) -> Option<Self> {
        if glib::check_version(2, 76, 0).is_some() {
            return None;
        }

        let display = widget.as_ref().display();
        let context = display.app_launch_context();

        // FIXME Call the vfunc directly since
        // g_app_launch_context_get_startup_notify_id has NULL checks.
        //
        // See https://gitlab.gnome.org/GNOME/glib/-/merge_requests/3933
        unsafe {
            let klass: *mut gtk4::gio::ffi::GAppLaunchContextClass =
                std::ptr::addr_of!((*context.class().parent()?)) as *mut _;
            let get_startup_notify_id = (*klass).get_startup_notify_id.as_ref()?;

            from_glib_full::<_, Option<String>>(get_startup_notify_id(
                context.as_ptr().cast(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ))
        }
        .map(Self::from)
    }
}
