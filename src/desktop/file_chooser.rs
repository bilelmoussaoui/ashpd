//! # Examples
//!
//! ## Opening a file
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{Choice, FileChooserProxy, FileFilter, OpenFileOptions};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!
//!     let proxy = FileChooserProxy::new(&connection).await?;
//!     let files = proxy
//!         .open_file(
//!             WindowIdentifier::default(),
//!             "open a file to read",
//!             OpenFileOptions::default()
//!                 .accept_label("read")
//!                 .modal(true)
//!                 .multiple(true)
//!                 .choice(
//!                     Choice::new("encoding", "Encoding", "latin15")
//!                         .insert("utf8", "Unicode (UTF-8)")
//!                         .insert("latin15", "Western"),
//!                 )
//!                 // A trick to have a checkbox
//!                 .choice(Choice::new("re-encode", "Re-encode", "false"))
//!                 .filter(FileFilter::new("SVG Image").mimetype("image/svg+xml")),
//!         )
//!         .await?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Ask to save a file
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{FileChooserProxy, FileFilter, SaveFileOptions};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = FileChooserProxy::new(&connection).await?;
//!     let files = proxy
//!         .save_file(
//!             WindowIdentifier::default(),
//!             "open a file to write",
//!             SaveFileOptions::default()
//!                 .accept_label("write")
//!                 .current_name("image.jpg")
//!                 .modal(true)
//!                 .filter(FileFilter::new("JPEG Image").glob("*.jpg")),
//!         )
//!         .await?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Ask to save multiple files
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{FileChooserProxy, SaveFilesOptions};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!
//!     let proxy = FileChooserProxy::new(&connection).await?;
//!     let files = proxy
//!         .save_files(
//!             WindowIdentifier::default(),
//!             "open files to write",
//!             SaveFilesOptions::default()
//!                 .accept_label("write files")
//!                 .modal(true)
//!                 .current_folder("/home/bilelmoussaoui/Pictures")
//!                 .files(&["test.jpg", "awesome.png"]),
//!         )
//!         .await?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::{
    helpers::{call_request_method, property},
    Error, HandleToken, WindowIdentifier,
};

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
/// A file filter, to limit the available file choices to a mimetype or a glob
/// pattern.
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Serialize_repr, Clone, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
#[doc(hidden)]
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

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
/// Presents the user with a choice to select from or as a checkbox.
pub struct Choice(String, String, Vec<(String, String)>, String);

impl Choice {
    /// Creates a new choice.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier of the choice.
    /// * `label` - user-visible name of the choice.
    /// * `initial_selection` - the initially selected value.
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
    pub fn id(&self) -> &str {
        &self.0
    }

    /// The user visible label of the choice.
    pub fn label(&self) -> &str {
        &self.1
    }

    /// The initially selected value.
    pub fn initial_selection(&self) -> &str {
        &self.3
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`FileChooserProxy::open_file`] request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Label for the accept button. Mnemonic underlines are allowed.
    accept_label: Option<String>,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// Whether multiple files can be selected or not.
    multiple: Option<bool>,
    /// Whether to select for folders instead of files.
    directory: Option<bool>,
    /// List of serialized file filters.
    filters: Vec<FileFilter>,
    /// Request that this filter be set by default at dialog creation.
    current_filter: Option<FileFilter>,
    /// List of serialized combo boxes to add to the file chooser
    choices: Vec<Choice>,
}

impl OpenFileOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = handle_token;
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
/// Specified options for a [`FileChooserProxy::save_file`] request.
pub struct SaveFileOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Label for the accept button. Mnemonic underlines are allowed.
    accept_label: Option<String>,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// Suggested filename.
    current_name: Option<String>,
    /// Suggested folder to save the file in.
    current_folder: Option<String>,
    /// The current file (when saving an existing file).
    current_file: Option<String>,
    /// List of serialized file filters.
    filters: Vec<FileFilter>,
    /// Request that this filter be set by default at dialog creation.
    current_filter: Option<FileFilter>,
    /// List of serialized combo boxes to add to the file chooser
    choices: Vec<Choice>,
}

impl SaveFileOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = handle_token;
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
/// Specified options for a [`FileChooserProxy::save_files`] request.
pub struct SaveFilesOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Label for the accept button. Mnemonic underlines are allowed.
    accept_label: Option<String>,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// List of serialized combo boxes to add to the file chooser
    choices: Vec<Choice>,
    /// Suggested folder to save the file in.
    current_folder: Option<String>,
    /// An array of file names to be saved.
    files: Option<Vec<String>>,
}

impl SaveFilesOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = handle_token;
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

    /// Specifies the current folder path.
    pub fn current_folder(mut self, current_folder: &str) -> Self {
        self.current_folder = Some(current_folder.into());
        self
    }

    /// Sets a list of files to save.
    pub fn files(mut self, files: &[&str]) -> Self {
        self.files = Some(files.to_vec().iter().map(|s| s.to_string()).collect());
        self
    }
}

#[derive(Debug, TypeDict, SerializeDict, Clone, DeserializeDict)]
/// A response to a [`FileChooserProxy::open_file`]/[`FileChooserProxy::save_file`]/[`FileChooserProxy::save_files`] request.
pub struct SelectedFiles {
    /// The selected files uris.
    pub uris: Vec<String>,
    /// The selected value of each choice as a tuple of (key, value)
    pub choices: Option<Vec<(String, String)>>,
}

/// The interface lets sandboxed applications ask the user for access to files
/// outside the sandbox. The portal backend will present the user with a file
/// chooser dialog.
#[derive(Debug)]
pub struct FileChooserProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> FileChooserProxy<'a> {
    /// Create a new instance of [`FileChooserProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<FileChooserProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.FileChooser")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Asks to open one or more files.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `title` - Title for the file chooser dialog.
    /// * `options` - [`OpenFileOptions`].
    pub async fn open_file(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<SelectedFiles, Error> {
        call_request_method(
            &self.0,
            &options.handle_token,
            "OpenFile",
            &(parent_window, title, &options),
        )
        .await
    }

    /// Asks for a location to save a file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `title` - Title for the file chooser dialog.
    /// * `options` - [`SaveFileOptions`].
    pub async fn save_file(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<SelectedFiles, Error> {
        call_request_method(
            &self.0,
            &options.handle_token,
            "SaveFile",
            &(parent_window, title, &options),
        )
        .await
    }

    /// Asks for a folder as a location to save one or more files.
    /// The names of the files will be used as-is and appended to the
    /// selected folder's path in the list of returned files.
    /// If the selected folder already contains a file with one of the given
    /// names, the portal may prompt or take some other action to
    /// construct a unique file name and return that instead.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `title` - Title for the file chooser dialog.
    /// * `options` - [`SaveFilesOptions`].
    pub async fn save_files(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<SelectedFiles, Error> {
        call_request_method(
            &self.0,
            &options.handle_token,
            "SaveFiles",
            &(parent_window, title, &options),
        )
        .await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}
