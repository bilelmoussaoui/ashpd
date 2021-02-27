/// Request access to the current logged user information such as the id, name
/// or their avatar uri.
pub mod account;
/// Request running an application in the background.
pub mod background;
/// Check if a camera is available, request access to it and open a PipeWire
/// remote stream.
pub mod camera;
/// Request access to specific devices such as camera, speakers or microphone.
pub mod device;
/// Compose an email.
pub mod email;
/// Open/save file(s) chooser.
pub mod file_chooser;
/// Enable/disable/query the status of Game Mode.
pub mod game_mode;
/// Inhibit the session from being restarted or the user from logging out.
pub mod inhibit;
/// Query the user's GPS location.
pub mod location;
/// Monitor memory level.
pub mod memory_monitor;
/// Check the status of the network on a user's machine.
pub mod network_monitor;
/// Send/withdraw notifications.
pub mod notification;
/// Open a file or a directory.
pub mod open_uri;
/// Print a docucment.
pub mod print;
/// Start a remote desktop session and interact with it.
pub mod remote_desktop;
/// Start a screencast session and get the PipeWire remote of it.
pub mod screencast;
/// Take a screenshot or pick a color.
pub mod screenshot;
/// Retrieve a per-application secret used to encrypt confidential data inside
/// the sandbox.
pub mod secret;
/// Read & listen to system settings changes.
pub mod settings;
/// Move a file to the trash.
pub mod trash;
/// Set a wallpaper on lockscreen, background or both.
pub mod wallpaper;
