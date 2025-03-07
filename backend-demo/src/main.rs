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

const NAME: &str = "org.freedesktop.impl.portal.desktop.ashpd-backend-demo";
// Run with
// RUST_LOG=ashpd_backend_demo=debug,ashpd=debug cargo run --manifest-path
// ./backend-demo/Cargo.toml

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    // Enable debug with `RUST_LOG=ashpd_backend_demo=debug COMMAND`.
    tracing_subscriber::fmt::init();

    ashpd::backend::Builder::new(NAME)?
        .account(Account)
        .screenshot(Screenshot)
        .secret(Secret)
        .settings(Settings::default())
        .wallpaper(Wallpaper)
        .build()
        .await?;

    loop {
        pending::<()>().await;
    }
}
