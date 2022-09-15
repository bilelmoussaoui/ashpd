use futures_util::future::pending;
mod account;
mod screenshot;
mod secret;
mod wallpaper;

use account::Account;
use screenshot::Screenshot;
use secret::Secret;
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

    tracing::debug!("Serving interfaces at {NAME}");
    let backend = ashpd::backend::Backend::new(NAME).await?;
    let account = ashpd::backend::account::Account::new(Account, &backend).await?;
    tokio::task::spawn(async move {
        loop {
            account.try_next().await.unwrap();
        }
    });
    let wallpaper = ashpd::backend::wallpaper::Wallpaper::new(Wallpaper, &backend).await?;
    tokio::task::spawn(async move {
        loop {
            wallpaper.try_next().await.unwrap();
        }
    });
    let screenshot = ashpd::backend::screenshot::Screenshot::new(Screenshot, &backend).await?;
    tokio::task::spawn(async move {
        loop {
            screenshot.try_next().await.unwrap();
        }
    });
    let secret = ashpd::backend::secret::Secret::new(Secret, &backend).await?;
    tokio::task::spawn(async move {
        loop {
            secret.try_next().await.unwrap();
        }
    });

    loop {
        pending::<()>().await;
    }
}
