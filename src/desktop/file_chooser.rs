use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options for a open file request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// Label for the accept button. Mnemonic underlines are allowed.
    pub accept_label: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: bool,
    /// Whether multiple files can be selected or not.
    pub multiple: bool,
    /// Whether to select for folders instead of files.
    pub directory: bool,
    // List of serialized file filters.
    // pub filters:
    // Request that this filter be set by default at dialog creation.
    // pub current_filter:
    // List of serialized combo boxes to add to the file chooser
    // pub choices:
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options for a save file request.
pub struct SaveFileOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// Label for the accept button. Mnemonic underlines are allowed.
    pub accept_label: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: bool,
    /// Suggested filename.
    pub current_name: Option<String>,
    /// Suggested folder to save the file in.
    pub current_folder: Vec<u8>,
    /// The current file (when saving an existing file).
    pub current_file: Vec<u8>,
    // List of serialized file filters.
    // pub filters:
    // Request that this filter be set by default at dialog creation.
    // pub current_filter:
    // List of serialized combo boxes to add to the file chooser
    // pub choices:
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options for a save files request.
pub struct SaveFilesOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// Label for the accept button. Mnemonic underlines are allowed.
    pub accept_label: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: bool,
    // List of serialized combo boxes to add to the file chooser
    // pub choices:
    /// Suggested folder to save the file in.
    pub current_folder: Vec<u8>,
    /// An array of file names to be saved.
    pub files: Vec<Vec<u8>>,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.FileChooser",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications ask the user for access to files outside the sandbox.
/// The portal backend will present the user with a file chooser dialog.
trait FileChooser {
    /// Asks to open one or more files.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the file chooser dialog
    /// * `options` - [`OpenFileOptions`]
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn open_file(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<String>;

    /// Asks for a location to save a file.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the file chooser dialog
    /// * `options` - [`SaveFileOptions`]
    ///
    /// [`SaveFileOptions`]: ./struct.SaveFileOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn save_file(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<String>;

    /// Asks for a folder as a location to save one or more files.
    /// The names of the files will be used as-is and appended to the
    /// selected folder's path in the list of returned files.
    /// If the selected folder already contains a file with one of the given
    /// names, the portal may prompt or take some other action to
    /// construct a unique file name and return that instead.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the file chooser dialog
    /// * `options` - [`SaveFilesOptions`]
    ///
    /// [`SaveFilesOptions`]: ./struct.SaveFilesOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn save_files(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}
