//! # Examples
//!
//! Opening a file
//! ```no_run
//! use ashpd::desktop::file_chooser::{
//!     Choice, FileChooserProxy, FileFilter, SelectedFiles, OpenFileOptions,
//! };
//! use ashpd::{RequestProxy, Response, WindowIdentifier};
//! use zbus::{fdo::Result, Connection};
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!
//!     let proxy = FileChooserProxy::new(&connection)?;
//!     let request_handle = proxy.open_file(
//!         WindowIdentifier::default(),
//!         "open a file to read",
//!         OpenFileOptions::default()
//!             .accept_label("read")
//!             .modal(true)
//!             .multiple(true)
//!             .choice(
//!                 Choice::new("encoding", "Encoding", "latin15")
//!                     .insert("utf8", "Unicode (UTF-8)")
//!                     .insert("latin15", "Western"),
//!             )
//!             // A trick to have a checkbox
//!             .choice(Choice::new("re-encode", "Re-encode", "false"))
//!             .filter(FileFilter::new("SVG Image").mimetype("image/svg+xml")),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|r: Response<SelectedFiles>| {
//!         println!("{:#?}", r.unwrap());
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Ask to save a file
//!
//! ```no_run
//! use ashpd::desktop::file_chooser::{
//!     FileChooserProxy, FileFilter, SelectedFiles, SaveFileOptions,
//! };
//! use ashpd::{RequestProxy, Response, WindowIdentifier};
//! use zbus::{fdo::Result, Connection};
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!
//!     let proxy = FileChooserProxy::new(&connection)?;
//!     let request_handle = proxy.save_file(
//!         WindowIdentifier::default(),
//!         "open a file to write",
//!         SaveFileOptions::default()
//!             .accept_label("write")
//!             .current_name("image.jpg")
//!             .modal(true)
//!             .filter(FileFilter::new("JPEG Image").glob("*.jpg")),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|r: Response<SelectedFiles>| {
//!         println!("{:#?}", r.unwrap());
//!     })?;
//!
//!     Ok(())
//! }
//!```
//!
//! Ask to save multiple files
//! ```no_run
//! use ashpd::desktop::file_chooser::{FileChooserProxy, SaveFilesOptions, SelectedFiles};
//! use ashpd::{RequestProxy, Response, WindowIdentifier};
//! use zbus::{fdo::Result, Connection};
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!
//!     let proxy = FileChooserProxy::new(&connection)?;
//!     let request_handle = proxy.save_files(
//!         WindowIdentifier::default(),
//!         "open files to write",
//!         SaveFilesOptions::default()
//!             .accept_label("write files")
//!             .modal(true)
//!             .current_folder("/home/bilelmoussaoui/Pictures")
//!             .files(vec!["test.jpg".to_string(), "awesome.png".to_string()]),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|r: Response<SelectedFiles>| {
//!         println!("{:#?}", r.unwrap());
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use crate::{HandleToken, NString, WindowIdentifier};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize, Deserialize, Type, Debug)]
/// A file filter, to limit the available file choices to a mimetype or a glob pattern.
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
enum FilterType {
    GlobPattern = 0,
    MimeType = 1,
}

impl FileFilter {
    /// Create a new file filter
    ///
    /// # Arguments
    ///
    /// * `label` - user-visible name of the file filter.
    pub fn new(label: &str) -> Self {
        Self(label.to_string(), vec![])
    }

    /// Adds a mime type to the file filter.
    pub fn mimetype(mut self, mimetype: &str) -> Self {
        self.1.push((FilterType::MimeType, mimetype.to_string()));
        self
    }

    /// Adds a glob pattern to the file filter.
    pub fn glob(mut self, pattern: &str) -> Self {
        self.1.push((FilterType::GlobPattern, pattern.to_string()));
        self
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
/// Presents the user with a choice to select from or as a checkbox.
pub struct Choice(String, String, Vec<(String, String)>, String);

impl Choice {
    /// Creates a new choice
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier of the choice
    /// * `label` - user-visible name of the choice
    /// * `initial_selection` - the initially selected value
    pub fn new(id: &str, label: &str, initial_selection: &str) -> Self {
        Self(
            id.to_string(),
            label.to_string(),
            vec![],
            initial_selection.to_string(),
        )
    }

    /// Adds a (key, value) as a choice.
    pub fn insert(mut self, key: &str, value: &str) -> Self {
        self.2.push((key.to_string(), value.to_string()));
        self
    }

    /// The choice's unique id
    pub fn id(&self) -> String {
        self.0.clone()
    }

    /// The user visible label of the choice.
    pub fn label(&self) -> String {
        self.1.clone()
    }

    /// The initially selected value.
    pub fn initial_selection(&self) -> String {
        self.3.clone()
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a `open_file` request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// Label for the accept button. Mnemonic underlines are allowed.
    pub accept_label: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// Whether multiple files can be selected or not.
    pub multiple: Option<bool>,
    /// Whether to select for folders instead of files.
    pub directory: Option<bool>,
    /// List of serialized file filters.
    pub filters: Vec<FileFilter>,
    /// Request that this filter be set by default at dialog creation.
    pub current_filter: Option<FileFilter>,
    /// List of serialized combo boxes to add to the file chooser
    pub choices: Vec<Choice>,
}

impl OpenFileOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets a user-visible string to the "accept" button.
    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Sets whether to allow multiple files selection.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    /// Sets whether to select directories or not.
    pub fn directory(mut self, directory: bool) -> Self {
        self.directory = Some(directory);
        self
    }

    /// Adds a files filter.
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Specifies the default filter.
    pub fn current_filter(mut self, current_filter: FileFilter) -> Self {
        self.current_filter = Some(current_filter);
        self
    }

    /// Adds a choice.
    pub fn choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a save file request.
pub struct SaveFileOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// Label for the accept button. Mnemonic underlines are allowed.
    pub accept_label: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// Suggested filename.
    pub current_name: Option<String>,
    /// Suggested folder to save the file in.
    pub current_folder: Option<NString>,
    /// The current file (when saving an existing file).
    pub current_file: Option<NString>,
    /// List of serialized file filters.
    pub filters: Vec<FileFilter>,
    /// Request that this filter be set by default at dialog creation.
    pub current_filter: Option<FileFilter>,
    /// List of serialized combo boxes to add to the file chooser
    pub choices: Vec<Choice>,
}

impl SaveFileOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets a user-visible string to the "accept" button.
    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    /// Sets the current file name.
    pub fn current_name(mut self, current_name: &str) -> Self {
        self.current_name = Some(current_name.to_string());
        self
    }

    /// Sets the current folder.
    pub fn current_folder(mut self, current_folder: &str) -> Self {
        self.current_folder = Some(current_folder.into());
        self
    }

    /// Sets the absolute path of the file.
    pub fn current_file(mut self, current_file: &str) -> Self {
        self.current_file = Some(current_file.into());
        self
    }

    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Adds a files filter.
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Sets the default filter.
    pub fn current_filter(mut self, current_filter: FileFilter) -> Self {
        self.current_filter = Some(current_filter);
        self
    }

    /// Adds a choice.
    pub fn choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a save files request.
pub struct SaveFilesOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// Label for the accept button. Mnemonic underlines are allowed.
    pub accept_label: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// List of serialized combo boxes to add to the file chooser
    pub choices: Vec<Choice>,
    /// Suggested folder to save the file in.
    pub current_folder: Option<NString>,
    /// An array of file names to be saved.
    pub files: Option<Vec<NString>>,
}

impl SaveFilesOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets a user-visible string to the "accept" button.
    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Adds a choice.
    pub fn choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }

    /// Specifices the current folder path.
    pub fn current_folder(mut self, current_folder: &str) -> Self {
        self.current_folder = Some(current_folder.into());
        self
    }

    /// Sets a list of files to save.
    pub fn files(mut self, files: Vec<String>) -> Self {
        self.files = Some(
            files
                .into_iter()
                .map(|f| f.into())
                .collect::<Vec<NString>>(),
        );
        self
    }
}

#[derive(Debug, TypeDict, SerializeDict, DeserializeDict)]
/// A response to an open/save file request.
pub struct SelectedFiles {
    /// The selected files uris.
    pub uris: Vec<String>,
    /// The selected value of each choice as a tuple of (key, value)
    pub choices: Option<Vec<(String, String)>>,
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
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the file chooser dialog
    /// * `options` - [`OpenFileOptions`]
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn open_file(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<OwnedObjectPath>;

    /// Asks for a location to save a file.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the file chooser dialog
    /// * `options` - [`SaveFileOptions`]
    ///
    /// [`SaveFileOptions`]: ./struct.SaveFileOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn save_file(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<OwnedObjectPath>;

    /// Asks for a folder as a location to save one or more files.
    /// The names of the files will be used as-is and appended to the
    /// selected folder's path in the list of returned files.
    /// If the selected folder already contains a file with one of the given
    /// names, the portal may prompt or take some other action to
    /// construct a unique file name and return that instead.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the file chooser dialog
    /// * `options` - [`SaveFilesOptions`]
    ///
    /// [`SaveFilesOptions`]: ./struct.SaveFilesOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn save_files(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}
