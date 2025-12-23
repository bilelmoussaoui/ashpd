mod application;

#[cfg(not(cargo_build))]
#[rustfmt::skip]
mod config;

#[cfg(cargo_build)]
#[path = "config_cargo.rs"]
mod config;

mod portals;
mod widgets;
mod window;

use application::Application;
use config::{GETTEXT_PACKAGE, LOCALEDIR};
use gettextrs::*;
use gtk::{gio, glib};
use gvdb_macros::include_gresource_from_xml;

use crate::portals::spawn_tokio_blocking;

static GRESOURCE_BYTES: &[u8] =
    include_gresource_from_xml!("demo/client/data/resources.gresource.xml");

fn main() -> glib::ExitCode {
    // Initialize logger, debug is carried out via debug!, info!, and warn!.
    tracing_subscriber::fmt::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).unwrap();
    textdomain(GETTEXT_PACKAGE).unwrap();

    gtk::glib::set_application_name(&gettext("ASHPD Demo"));
    gst::init().expect("Unable to init gstreamer");

    gst4gtk::plugin_register_static().expect("Failed to register gstgtk4 plugin");

    let res = gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES)).unwrap();
    gio::resources_register(&res);

    spawn_tokio_blocking(async move {
        if let Err(err) = ashpd::register_host_app(config::APP_ID.try_into().unwrap()).await {
            tracing::warn!("Failed to register host app: {err}");
        }
    });

    let mut args = std::env::args();
    if args.any(|x| x == "--replace") {
        spawn_tokio_blocking(async move {
            if let Err(err) = Application::stop_current_instance().await {
                tracing::error!("Failed to replace current instance {}", err);
            };
        });
    }

    Application::run()
}
