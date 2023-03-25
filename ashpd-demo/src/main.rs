mod application;
#[rustfmt::skip]
mod config;
mod portals;
mod widgets;
mod window;

use application::Application;
use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::*;
use gtk::{gio, glib};

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

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let mut args = std::env::args();
    if args.any(|x| x == "--replace") {
        if let Err(err) = Application::stop_current_instance() {
            tracing::error!("Failed to replace current instance {}", err);
        };
    }

    let app = Application::new();
    app.run()
}
