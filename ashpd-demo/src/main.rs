mod application;
#[rustfmt::skip]
mod config;
mod portals;
mod sidebar_row;
mod widgets;
mod window;

use application::ExampleApplication;
use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::*;
use gtk::gio;

fn main() {
    // Initialize logger, debug is carried out via debug!, info!, and warn!.
    pretty_env_logger::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).unwrap();
    textdomain(GETTEXT_PACKAGE).unwrap();

    gtk::glib::set_application_name("ASHPD Demo");
    gtk::init().expect("Unable to start GTK4");
    gst::init().expect("Unable to init gstreamer");

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = ExampleApplication::new();
    app.run();
}
