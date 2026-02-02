pub type Result<T> = std::result::Result<T, crate::error::PortalError>;

#[cfg(feature = "backend_access")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_access")))]
pub mod access;
#[cfg(feature = "backend_account")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_account")))]
pub mod account;
#[cfg(feature = "backend_app_chooser")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_app_chooser")))]
pub mod app_chooser;
#[cfg(feature = "backend_background")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_background")))]
pub mod background;
mod builder;
pub use builder::Builder;
#[cfg(feature = "backend_email")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_email")))]
pub mod email;
#[cfg(feature = "backend_file_chooser")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_file_chooser")))]
pub mod file_chooser;
#[cfg(feature = "backend_lockdown")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_lockdown")))]
pub mod lockdown;
#[cfg(feature = "backend_permission_store")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_permission_store")))]
pub mod permission_store;
#[cfg(feature = "backend_print")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_print")))]
pub mod print;
pub mod request;
#[cfg(feature = "backend_screencast")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_screencast")))]
pub mod screencast;
#[cfg(feature = "backend_screenshot")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_screenshot")))]
pub mod screenshot;
#[cfg(feature = "backend_secret")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_secret")))]
pub mod secret;
pub mod session;
#[cfg(feature = "backend_settings")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_settings")))]
pub mod settings;
mod spawn;
#[cfg(feature = "backend_usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_usb")))]
pub mod usb;
#[cfg(feature = "backend_wallpaper")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend_wallpaper")))]
pub mod wallpaper;
