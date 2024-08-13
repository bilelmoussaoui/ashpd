pub type Result<T> = std::result::Result<T, crate::error::PortalError>;

pub mod access;
pub mod account;
pub mod email;
pub mod lockdown;
pub mod print;
pub mod request;
pub mod screenshot;
pub mod secret;
pub mod settings;
pub mod wallpaper;
