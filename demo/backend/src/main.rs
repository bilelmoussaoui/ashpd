use futures_util::future::pending;
mod access;
mod account;
mod screencast;
mod screenshot;
mod secret;
mod settings;
mod wallpaper;

use access::Access;
use account::Account;
use screencast::Screencast;
use screenshot::Screenshot;
use secret::Secret;
use settings::Settings;
use wallpaper::Wallpaper;

const NAME: &str = "org.freedesktop.impl.portal.desktop.ashpd-backend-demo";
// Run with
// RUST_LOG=ashpd_backend_demo=debug,ashpd=debug cargo run --manifest-path
// ./demo/backend/Cargo.toml

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    // Enable debug with `RUST_LOG=ashpd_backend_demo=debug COMMAND`.
    tracing_subscriber::fmt::init();

    ashpd::backend::Builder::new(NAME)?
        .access(Access)
        .account(Account)
        .screencast(Screencast::default())
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
