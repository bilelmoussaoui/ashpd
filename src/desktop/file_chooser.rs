//! # Examples
//!
//! ## Opening a file
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{Choice, FileChooserProxy, FileFilter, OpenFileOptions};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!
//!     let proxy = FileChooserProxy::new(&connection).await?;
//!     let files = proxy
//!         .open_file(
//!             &WindowIdentifier::default(),
//!             "open a file to read",
//!             OpenFileOptions::default()
//!                 .accept_label("read")
//!                 .modal(true)
//!                 .multiple(true)
//!                 .add_choice(
//!                     Choice::new("encoding", "Encoding", "latin15")
//!                         .insert("utf8", "Unicode (UTF-8)")
//!                         .insert("latin15", "Western"),
//!                 )
//!                 // A trick to have a checkbox
//!                 .add_choice(Choice::boolean("re-encode", "Re-encode", false))
//!                 .add_filter(FileFilter::new("SVG Image").mimetype("image/svg+xml")),
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
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = FileChooserProxy::new(&connection).await?;
//!     let files = proxy
//!         .save_file(
//!             &WindowIdentifier::default(),
//!             "open a file to write",
//!             SaveFileOptions::default()
//!                 .accept_label("write")
//!                 .current_name("image.jpg")
//!                 .modal(true)
//!                 .add_filter(FileFilter::new("JPEG Image").glob("*.jpg")),
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
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!
//!     let proxy = FileChooserProxy::new(&connection).await?;
//!     let files = proxy
//!         .save_files(
//!             &WindowIdentifier::default(),
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
use std::os::unix::ffi::OsStrExt;
use std::{ffi::CString, path::Path};
use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_request_method, Error, WindowIdentifier};

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
/// A file filter, to limit the available file choices to a mimetype or a glob
/// pattern.
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Serialize_repr, Clone, Deserialize_repr, PartialEq, Debug, Type)]
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
    #[must_use]
    pub fn mimetype(mut self, mimetype: &str) -> Self {
        self.1.push((FilterType::MimeType, mimetype.to_string()));
        self
    }

    /// Adds a glob pattern to the file filter.
    #[must_use]
    pub fn glob(mut self, pattern: &str) -> Self {
        self.1.push((FilterType::GlobPattern, pattern.to_string()));
        self
    }
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
/// Presents the user with a choice to select from or as a checkbox.
pub struct Choice(String, String, Vec<(String, String)>, String);

impl Choice {
    /// Creates a checkbox choice.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier of the choice.
    /// * `label` - user-visible name of the choice.
    /// * `state` - the initial state value.
    pub fn boolean(id: &str, label: &str, state: bool) -> Self {
        Self::new(id, label, &state.to_string())
    }

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
    #[must_use]
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

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`FileChooserProxy::open_file`] request.
#[zvariant(signature = "dict")]
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
    /// Sets a user-visible string to the "accept" button.
    #[must_use]
    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Sets whether to allow multiple files selection.
    #[must_use]
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    /// Sets whether to select directories or not.
    #[must_use]
    pub fn directory(mut self, directory: bool) -> Self {
        self.directory = Some(directory);
        self
    }

    /// Adds a files filter.
    #[must_use]
    pub fn add_filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Specifies the default filter.
    #[must_use]
    pub fn current_filter(mut self, current_filter: FileFilter) -> Self {
        self.current_filter = Some(current_filter);
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn add_choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`FileChooserProxy::save_file`] request.
#[zvariant(signature = "dict")]
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
    current_folder: Option<Vec<u8>>,
    /// The current file (when saving an existing file).
    current_file: Option<Vec<u8>>,
    /// List of serialized file filters.
    filters: Vec<FileFilter>,
    /// Request that this filter be set by default at dialog creation.
    current_filter: Option<FileFilter>,
    /// List of serialized combo boxes to add to the file chooser
    choices: Vec<Choice>,
}

impl SaveFileOptions {
    /// Sets a user-visible string to the "accept" button.
    #[must_use]
    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    /// Sets the current file name.
    #[must_use]
    pub fn current_name(mut self, current_name: &str) -> Self {
        self.current_name = Some(current_name.to_string());
        self
    }

    /// Sets the current folder.
    #[must_use]
    pub fn current_folder(mut self, current_folder: impl AsRef<Path>) -> Self {
        let cstr = CString::new(current_folder.as_ref().as_os_str().as_bytes())
            .expect("`current_folder` should not be null terminated");
        self.current_folder = Some(cstr.into_bytes_with_nul());
        self
    }

    /// Sets the absolute path of the file.
    #[must_use]
    pub fn current_file(mut self, current_file: impl AsRef<Path>) -> Self {
        let cstr = CString::new(current_file.as_ref().as_os_str().as_bytes())
            .expect("`current_file` should not be null terminated");
        self.current_file = Some(cstr.into_bytes_with_nul());
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Adds a files filter.
    #[must_use]
    pub fn add_filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Sets the default filter.
    #[must_use]
    pub fn current_filter(mut self, current_filter: FileFilter) -> Self {
        self.current_filter = Some(current_filter);
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn add_choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`FileChooserProxy::save_files`] request.
#[zvariant(signature = "dict")]
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
    current_folder: Option<Vec<u8>>,
    /// An array of file names to be saved.
    files: Option<Vec<Vec<u8>>>,
}

impl SaveFilesOptions {
    /// Sets a user-visible string to the "accept" button.
    #[must_use]
    pub fn accept_label(mut self, accept_label: &str) -> Self {
        self.accept_label = Some(accept_label.to_string());
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn add_choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }

    /// Specifies the current folder path.
    #[must_use]
    pub fn current_folder(mut self, current_folder: impl AsRef<Path>) -> Self {
        let cstr = CString::new(current_folder.as_ref().as_os_str().as_bytes())
            .expect("`current_folder` should not be null terminated");
        self.current_folder = Some(cstr.into_bytes_with_nul());
        self
    }

    /// Sets a list of files to save.
    #[must_use]
    pub fn files(mut self, files: &[impl AsRef<Path>]) -> Self {
        self.files = Some(
            files
                .iter()
                .map(|s| {
                    let cstr = CString::new(s.as_ref().as_os_str().as_bytes())
                        .expect("`files` should not be null terminated");
                    cstr.into_bytes_with_nul()
                })
                .collect(),
        );
        self
    }
}

#[derive(Debug, Type, SerializeDict, Clone, DeserializeDict)]
/// A response to a
/// [`FileChooserProxy::open_file`]/[`FileChooserProxy::save_file`]/
/// [`FileChooserProxy::save_files`] request.
#[zvariant(signature = "dict")]
pub struct SelectedFiles {
    uris: Vec<String>,
    choices: Option<Vec<(String, String)>>,
}

impl SelectedFiles {
    /// The selected files uris.
    pub fn uris(&self) -> &[String] {
        self.uris.as_slice()
    }

    /// The selected value of each choice as a tuple of (key, value)
    pub fn choices(&self) -> &[(String, String)] {
        self.choices.as_deref().unwrap_or_default()
    }
}

/// The interface lets sandboxed applications ask the user for access to files
/// outside the sandbox. The portal backend will present the user with a file
/// chooser dialog.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.FileChooser`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.FileChooser).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.FileChooser")]
pub struct FileChooserProxy<'a>(zbus::Proxy<'a>);

impl<'a> FileChooserProxy<'a> {
    /// Create a new instance of [`FileChooserProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<FileChooserProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.FileChooser")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Asks to open one or more files.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `title` - Title for the file chooser dialog.
    /// * `options` - [`OpenFileOptions`].
    ///
    /// # Specifications
    ///
    /// See also [`OpenFile`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileChooser.OpenFile).
    #[doc(alias = "OpenFile")]
    #[doc(alias = "xdp_portal_open_file")]
    pub async fn open_file(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<SelectedFiles, Error> {
        call_request_method(
            self.inner(),
            &options.handle_token,
            "OpenFile",
            &(&identifier, title, &options),
        )
        .await
    }

    /// Asks for a location to save a file.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `title` - Title for the file chooser dialog.
    /// * `options` - [`SaveFileOptions`].
    ///
    /// # Specifications
    ///
    /// See also [`SaveFile`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileChooser.SaveFile).
    #[doc(alias = "SaveFile")]
    #[doc(alias = "xdp_portal_save_file")]
    pub async fn save_file(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<SelectedFiles, Error> {
        call_request_method(
            self.inner(),
            &options.handle_token,
            "SaveFile",
            &(&identifier, title, &options),
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
    /// * `identifier` - Identifier for the application window.
    /// * `title` - Title for the file chooser dialog.
    /// * `options` - [`SaveFilesOptions`].
    ///
    /// # Specifications
    ///
    /// See also [`SaveFiles`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileChooser.SaveFiles).
    #[doc(alias = "SaveFiles")]
    #[doc(alias = "xdp_portal_save_files")]
    pub async fn save_files(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<SelectedFiles, Error> {
        call_request_method(
            self.inner(),
            &options.handle_token,
            "SaveFiles",
            &(&identifier, title, &options),
        )
        .await
    }
}
