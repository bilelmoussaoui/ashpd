use futures_util::future::pending;
mod account;
mod screenshot;
mod secret;
mod settings;
mod wallpaper;

use account::Account;
use screenshot::Screenshot;
use secret::Secret;
use settings::Settings;
use wallpaper::Wallpaper;

// NOTE Uncomment if you have ashpd-backend-demo.portal installed.
// const NAME: &str = "org.freedesktop.impl.portal.desktop.ashpd-backend-demo";
const NAME: &str = "org.freedesktop.impl.portal.desktop.gnome";
// Run with
// RUST_LOG=ashpd_backend_demo=debug,ashpd=debug cargo run --manifest-path
// ./backend-demo/Cargo.toml

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    // Enable debug with `RUST_LOG=ashpd_backend_demo=debug COMMAND`.
    tracing_subscriber::fmt::init();

    let cnx = zbus::connection::Builder::session()?
        .name(NAME)?
        .build()
        .await?;
    let object_server = cnx.object_server();

    let portal = ashpd::backend::account::AccountInterface::new(Account, cnx.clone());
    tracing::debug!("Serving interface `org.freedesktop.impl.portal.Account`");
    object_server
        .at("/org/freedesktop/portal/desktop", portal)
        .await?;

    let portal = ashpd::backend::screenshot::ScreenshotInterface::new(Screenshot, cnx.clone());
    tracing::debug!("Serving interface `org.freedesktop.impl.portal.Screenshot`");
    object_server
        .at("/org/freedesktop/portal/desktop", portal)
        .await?;

    let portal = ashpd::backend::secret::SecretInterface::new(Secret, cnx.clone());
    tracing::debug!("Serving interface `org.freedesktop.impl.portal.Secret`");
    object_server
        .at("/org/freedesktop/portal/desktop", portal)
        .await?;

    let portal = ashpd::backend::settings::SettingsInterface::new(Settings::default(), cnx.clone());
    tracing::debug!("Serving interface `org.freedesktop.impl.portal.Settings`");
    object_server
        .at("/org/freedesktop/portal/desktop", portal)
        .await?;

    let portal = ashpd::backend::wallpaper::WallpaperInterface::new(Wallpaper, cnx.clone());
    tracing::debug!("Serving interface `org.freedesktop.impl.portal.Wallpaper`");
    object_server
        .at("/org/freedesktop/portal/desktop", portal)
        .await?;

    loop {
        pending::<()>().await;
    }
}
