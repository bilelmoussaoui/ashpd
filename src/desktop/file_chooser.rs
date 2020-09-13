//! # Examples
//!
//! Opening a file
//! ```no_run
//! use libportal::desktop::file_chooser::{
//!     Choice, FileChooserProxy, FileFilter, FilterType, SelectedFiles, OpenFileOptions,
//! };
//! use libportal::{RequestProxy, Response, WindowIdentifier};
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
//!                     .add("utf8", "Unicode (UTF-8)")
//!                     .add("latin15", "Western"),
//!             )
//!             // A trick to have a checkbox
//!             .choice(Choice::new("reencode", "Reencode", "false"))
//!             .filter(FileFilter::new("SVG Image").add(FilterType::MimeType, "image/svg+xml")),
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
//! use libportal::desktop::file_chooser::{
//!     FileChooserProxy, FileFilter, FilterType, SelectedFiles, SaveFileOptions,
//! };
//! use libportal::{RequestProxy, Response, WindowIdentifier};
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
//!             .filter(FileFilter::new("JPEG Image").add(FilterType::GlobPattern, "*.jpg")),
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
//! use libportal::desktop::file_chooser::{FileChooserProxy, SaveFilesOptions, SelectedFiles};
//! use libportal::{RequestProxy, Response, WindowIdentifier};
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
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum FilterType {
    GlobPattern = 0,
    MimeType = 1,
}

impl FileFilter {
    pub fn new(label: &str) -> Self {
        Self(label.to_string(), vec![])
    }

    pub fn add(mut self, filter_type: FilterType, filter: &str) -> Self {
        self.1.push((filter_type, filter.to_string()));
        self
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Choice(String, String, Vec<(String, String)>, String);

impl Choice {
    pub fn new(id: &str, label: &str, initial_selection: &str) -> Self {
        Self(
            id.to_string(),
            label.to_string(),
            vec![],
            initial_selection.to_string(),
        )
    }

    pub fn add(mut self, key: &str, value: &str) -> Self {
        self.2.push((key.to_string(), value.to_string()));
        self
    }

    pub fn id(&self) -> String {
        self.0.clone()
    }

    pub fn label(&self) -> String {
        self.1.clone()
    }

    pub fn initial_selection(&self) -> String {
        self.3.clone()
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a open file request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
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
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    pub fn directory(mut self, directory: bool) -> Self {
        self.directory = Some(directory);
        self
    }

    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn current_filter(mut self, current_filter: FileFilter) -> Self {
        self.current_filter = Some(current_filter);
        self
    }

    pub fn choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a save file request.
pub struct SaveFileOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
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
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    pub fn current_name(mut self, current_name: &str) -> Self {
        self.current_name = Some(current_name.to_string());
        self
    }

    pub fn current_folder(mut self, current_folder: &str) -> Self {
        self.current_folder = Some(current_folder.into());
        self
    }

    pub fn current_file(mut self, current_file: &str) -> Self {
        self.current_file = Some(current_file.into());
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn current_filter(mut self, current_filter: FileFilter) -> Self {
        self.current_filter = Some(current_filter);
        self
    }

    pub fn choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a save files request.
pub struct SaveFilesOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
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
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    pub fn choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }

    pub fn current_folder(mut self, current_folder: &str) -> Self {
        self.current_folder = Some(current_folder.into());
        self
    }

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
pub struct SelectedFiles {
    pub uris: Vec<String>,
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
