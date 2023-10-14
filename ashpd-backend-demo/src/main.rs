use std::{cell::RefCell, rc::Rc, sync::Arc};

use ashpd::backend::Backend;
use gettextrs::LocaleCategory;
use gtk::glib;
//#[allow(deprecated)] // We use the deprecated GTK APIs
// mod file_chooser;
mod account;
mod account_preview;
mod screenshot;
mod settings;
mod wallpaper;
mod wallpaper_preview;

// use file_chooser::FileChooser;
use account::Account;
use screenshot::Screenshot;
use settings::Settings;
use wallpaper::Wallpaper;

// NOTE Uncomment if you have ashpd-backend-demo.portal installed.
const NAME: &str = "org.freedesktop.impl.portal.desktop.ashpd-backend-demo";
// const NAME: &str = "org.freedesktop.impl.portal.desktop.gnome";

fn main() {
    // Enable debug with `RUST_LOG=xdp_ashpd_gnome=debug COMMAND`.
    tracing_subscriber::fmt::init();

    // FIXME Use meson here
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain("ashpd-backend-demo", "/usr/share/locale")
        .expect("Unable to bind the text domain");
    gettextrs::textdomain("ashpd-backend-demo").expect("Unable to switch to the text domain");

    glib::set_prgname(Some("ashpd-backend-demo"));

    // Avoid pointless and confusing recursion
    glib::unsetenv("GTK_USE_PORTAL");
    glib::setenv("ADW_DISABLE_PORTAL", "1", true).unwrap();
    glib::setenv("GSK_RENDERER", "cairo", true).unwrap();

    gtk::init().unwrap();
    adw::init().unwrap();

    log::debug!("Starting interfaces at {NAME}");

    let main_loop = glib::MainLoop::new(None, false);
    let ctx = glib::MainContext::default();
    let backend = Rc::new(RefCell::new(None));
    ctx.spawn_local(async move {
        let res = init_interfaces().await.unwrap();
        backend.borrow_mut().replace(Some(res));
    });

    log::debug!("Starting Main Loop");
    // TODO Ideally we do something like
    // https://github.com/gtk-rs/gtk-rs-core/blob/master/examples/gio_async_tls/main.rs
    main_loop.run();
}

async fn init_interfaces() -> Result<Backend, ashpd::Error> {
    let backend = Backend::new(NAME.to_string()).await?;

    let wallpaper =
        Arc::new(ashpd::backend::wallpaper::Wallpaper::new(Wallpaper::default(), &backend).await?);
    let screenshot = Arc::new(
        ashpd::backend::screenshot::Screenshot::new(Screenshot::default(), &backend).await?,
    );
    let account =
        Arc::new(ashpd::backend::account::Account::new(Account::default(), &backend).await?);
    let settings =
        Arc::new(ashpd::backend::settings::Settings::new(Settings::default(), &backend).await?);
    // let file_chooser = Arc::new(
    //    ashpd::backend::file_chooser::FileChooser::new(FileChooser::new(sender),
    // &backend).await?,
    //);

    let imp = Arc::clone(&wallpaper);

    glib::MainContext::default().spawn_local(async move {
        loop {
            if let Err(err) = imp.try_next().await {
                log::error!("Could not handle wallpaper: {err:?}");
            }
        }
    });

    let imp = Arc::clone(&account);
    glib::MainContext::default().spawn_local(async move {
        loop {
            if let Err(err) = imp.try_next().await {
                log::error!("Could not handle wallpaper: {err:?}");
            }
        }
    });

    let imp = Arc::clone(&screenshot);
    glib::MainContext::default().spawn_local(async move {
        loop {
            if let Err(err) = imp.try_next().await {
                log::error!("Could not handle wallpaper: {err:?}");
            }
        }
    });

    let imp = Arc::clone(&settings);
    glib::MainContext::default().spawn_local(async move {
        loop {
            if let Err(err) = imp.try_next().await {
                log::error!("Could not handle wallpaper: {err:?}");
            }
        }
    });

    // if let Some(action) = file_chooser.try_next() {
    // let imp = Arc::clone(&file_chooser);
    // async_std::task::spawn(async move {
    // if let Err(err) = imp.activate(action).await {
    // log::error!("Could not handle file chooser: {err:?}");
    // }
    // });
    // };
    Ok(backend)
}
