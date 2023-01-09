//! The interface lets sandboxed applications ask the user for access to files
//! outside the sandbox. The portal backend will present the user with a file
//! chooser dialog.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.FileChooser`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.FileChooser).
//!
//! ### Examples
//!
//! #### Opening a file
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{Choice, FileFilter, OpenFileRequest};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let files = OpenFileRequest::default()
//!         .title("open a file to read")
//!         .accept_label("read")
//!         .modal(true)
//!         .multiple(true)
//!         .choice(
//!             Choice::new("encoding", "Encoding", "latin15")
//!                 .insert("utf8", "Unicode (UTF-8)")
//!                 .insert("latin15", "Western"),
//!         )
//!         // A trick to have a checkbox
//!         .choice(Choice::boolean("re-encode", "Re-encode", false))
//!         .filter(FileFilter::new("SVG Image").mimetype("image/svg+xml"))
//!         .build()
//!         .await?
//!         .response()?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```
//!
//! #### Ask to save a file
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{FileFilter, SaveFileRequest};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let files = SaveFileRequest::default()
//!         .title("open a file to write")
//!         .accept_label("write")
//!         .current_name("image.jpg")
//!         .modal(true)
//!         .filter(FileFilter::new("JPEG Image").glob("*.jpg"))
//!         .build()
//!         .await?
//!         .response()?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```
//!
//! #### Ask to save multiple files
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::SaveFilesRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let files = SaveFilesRequest::default()
//!         .title("open files to write")
//!         .accept_label("write files")
//!         .modal(true)
//!         .current_folder("/home/bilelmoussaoui/Pictures")
//!         .files(&["test.jpg", "awesome.png"])
//!         .build()
//!         .await?
//!         .response()?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```

use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, Request};
use crate::{proxy::Proxy, Error, WindowIdentifier};

#[derive(Clone, Serialize, Type, Debug)]
/// A file filter, to limit the available file choices to a mimetype or a glob
/// pattern.
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Clone, Serialize_repr, Debug, Type)]
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
        Self(label.to_owned(), vec![])
    }

    /// Adds a mime type to the file filter.
    #[must_use]
    pub fn mimetype(mut self, mimetype: &str) -> Self {
        self.1.push((FilterType::MimeType, mimetype.to_owned()));
        self
    }

    /// Adds a glob pattern to the file filter.
    #[must_use]
    pub fn glob(mut self, pattern: &str) -> Self {
        self.1.push((FilterType::GlobPattern, pattern.to_owned()));
        self
    }
}

#[derive(Clone, Serialize, Deserialize, Type, Debug)]
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
            id.to_owned(),
            label.to_owned(),
            vec![],
            initial_selection.to_owned(),
        )
    }

    /// Adds a (key, value) as a choice.
    #[must_use]
    pub fn insert(mut self, key: &str, value: &str) -> Self {
        self.2.push((key.to_owned(), value.to_owned()));
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

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct OpenFileOptions {
    handle_token: HandleToken,
    accept_label: Option<String>,
    modal: Option<bool>,
    multiple: Option<bool>,
    directory: Option<bool>,
    filters: Vec<FileFilter>,
    current_filter: Option<FileFilter>,
    choices: Vec<Choice>,
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct SaveFileOptions {
    handle_token: HandleToken,
    accept_label: Option<String>,
    modal: Option<bool>,
    current_name: Option<String>,
    current_folder: Option<Vec<u8>>,
    current_file: Option<Vec<u8>>,
    filters: Vec<FileFilter>,
    current_filter: Option<FileFilter>,
    choices: Vec<Choice>,
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct SaveFilesOptions {
    handle_token: HandleToken,
    accept_label: Option<String>,
    modal: Option<bool>,
    choices: Vec<Choice>,
    current_folder: Option<Vec<u8>>,
    files: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Type, DeserializeDict)]
/// A response of [`OpenFileRequest`], [`SaveFileRequest`] or
/// [`SaveFilesRequest`].
#[zvariant(signature = "dict")]
pub struct SelectedFiles {
    uris: Vec<url::Url>,
    choices: Option<Vec<(String, String)>>,
}

impl SelectedFiles {
    /// The selected files uris.
    pub fn uris(&self) -> &[url::Url] {
        self.uris.as_slice()
    }

    /// The selected value of each choice as a tuple of (key, value)
    pub fn choices(&self) -> &[(String, String)] {
        self.choices.as_deref().unwrap_or_default()
    }
}

#[doc(alias = "org.freedesktop.portal.FileChooser")]
struct FileChooserProxy<'a>(Proxy<'a>);

impl<'a> FileChooserProxy<'a> {
    /// Create a new instance of [`FileChooserProxy`].
    pub async fn new() -> Result<FileChooserProxy<'a>, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.FileChooser").await?;
        Ok(Self(proxy))
    }

    pub async fn open_file(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<Request<SelectedFiles>, Error> {
        self.0
            .request(
                &options.handle_token,
                "OpenFile",
                &(&identifier, title, &options),
            )
            .await
    }

    pub async fn save_file(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<Request<SelectedFiles>, Error> {
        self.0
            .request(
                &options.handle_token,
                "SaveFile",
                &(&identifier, title, &options),
            )
            .await
    }

    pub async fn save_files(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<Request<SelectedFiles>, Error> {
        self.0
            .request(
                &options.handle_token,
                "SaveFiles",
                &(&identifier, title, &options),
            )
            .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_open_file")]
pub struct OpenFileRequest {
    identifier: WindowIdentifier,
    title: String,
    options: OpenFileOptions,
}

impl OpenFileRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    /// Sets a title for the file chooser dialog.
    #[must_use]
    pub fn title<'a>(mut self, title: impl Into<Option<&'a str>>) -> Self {
        self.title = title.into().map(ToOwned::to_owned).unwrap_or_default();
        self
    }

    /// Sets a user-visible string to the "accept" button.
    #[must_use]
    pub fn accept_label<'a>(mut self, accept_label: impl Into<Option<&'a str>>) -> Self {
        self.options.accept_label = accept_label.into().map(ToOwned::to_owned);
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: impl Into<Option<bool>>) -> Self {
        self.options.modal = modal.into();
        self
    }

    /// Sets whether to allow multiple files selection.
    #[must_use]
    pub fn multiple(mut self, multiple: impl Into<Option<bool>>) -> Self {
        self.options.multiple = multiple.into();
        self
    }

    /// Sets whether to select directories or not.
    #[must_use]
    pub fn directory(mut self, directory: impl Into<Option<bool>>) -> Self {
        self.options.directory = directory.into();
        self
    }

    /// Adds a files filter.
    #[must_use]
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.options.filters.push(filter);
        self
    }

    #[must_use]
    pub fn filters(mut self, filters: impl IntoIterator<Item = FileFilter>) -> Self {
        self.options.filters = filters.into_iter().collect();
        self
    }

    /// Specifies the default filter.
    #[must_use]
    pub fn current_filter(mut self, current_filter: impl Into<Option<FileFilter>>) -> Self {
        self.options.current_filter = current_filter.into();
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn choice(mut self, choice: Choice) -> Self {
        self.options.choices.push(choice);
        self
    }

    #[must_use]
    pub fn choices(mut self, choices: impl IntoIterator<Item = Choice>) -> Self {
        self.options.choices = choices.into_iter().collect();
        self
    }

    pub async fn build(self) -> Result<Request<SelectedFiles>, Error> {
        let proxy = FileChooserProxy::new().await?;
        proxy
            .open_file(&self.identifier, &self.title, self.options)
            .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_save_files")]
pub struct SaveFilesRequest {
    identifier: WindowIdentifier,
    title: String,
    options: SaveFilesOptions,
}

impl SaveFilesRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    /// Sets a title for the file chooser dialog.
    #[must_use]
    pub fn title<'a>(mut self, title: impl Into<Option<&'a str>>) -> Self {
        self.title = title.into().map(ToOwned::to_owned).unwrap_or_default();
        self
    }

    /// Sets a user-visible string to the "accept" button.
    #[must_use]
    pub fn accept_label<'a>(mut self, accept_label: impl Into<Option<&'a str>>) -> Self {
        self.options.accept_label = accept_label.into().map(ToOwned::to_owned);
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: impl Into<Option<bool>>) -> Self {
        self.options.modal = modal.into();
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn choice(mut self, choice: Choice) -> Self {
        self.options.choices.push(choice);
        self
    }

    #[must_use]
    pub fn choices(mut self, choices: impl IntoIterator<Item = Choice>) -> Self {
        self.options.choices = choices.into_iter().collect();
        self
    }

    /// Specifies the current folder path.
    #[must_use]
    pub fn current_folder<P: AsRef<Path>>(mut self, current_folder: impl Into<Option<P>>) -> Self {
        self.options.current_folder = current_folder.into().map(|c| {
            CString::new(c.as_ref().as_os_str().as_bytes())
                .expect("`current_folder` should not be null terminated")
                .into_bytes_with_nul()
        });
        self
    }

    /// Sets a list of files to save.
    #[must_use]
    pub fn files<P: IntoIterator<Item = impl AsRef<Path>>>(
        mut self,
        files: impl Into<Option<P>>,
    ) -> Self {
        self.options.files = files.into().map(|files| {
            files
                .into_iter()
                .map(|s| {
                    let cstr = CString::new(s.as_ref().as_os_str().as_bytes())
                        .expect("`files` should not be null terminated");
                    cstr.into_bytes_with_nul()
                })
                .collect()
        });
        self
    }

    pub async fn build(self) -> Result<Request<SelectedFiles>, Error> {
        let proxy = FileChooserProxy::new().await?;
        proxy
            .save_files(&self.identifier, &self.title, self.options)
            .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_save_file")]
pub struct SaveFileRequest {
    identifier: WindowIdentifier,
    title: String,
    options: SaveFileOptions,
}

impl SaveFileRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    /// Sets a title for the file chooser dialog.
    #[must_use]
    pub fn title<'a>(mut self, title: impl Into<Option<&'a str>>) -> Self {
        self.title = title.into().map(ToOwned::to_owned).unwrap_or_default();
        self
    }

    /// Sets a user-visible string to the "accept" button.
    #[must_use]
    pub fn accept_label<'a>(mut self, accept_label: impl Into<Option<&'a str>>) -> Self {
        self.options.accept_label = accept_label.into().map(ToOwned::to_owned);
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: impl Into<Option<bool>>) -> Self {
        self.options.modal = modal.into();
        self
    }

    /// Sets the current file name.
    #[must_use]
    pub fn current_name<'a>(mut self, current_name: impl Into<Option<&'a str>>) -> Self {
        self.options.current_name = current_name.into().map(ToOwned::to_owned);
        self
    }

    /// Sets the current folder.
    #[must_use]
    pub fn current_folder<P: AsRef<Path>>(mut self, current_folder: impl Into<Option<P>>) -> Self {
        self.options.current_folder = current_folder.into().map(|c| {
            CString::new(c.as_ref().as_os_str().as_bytes())
                .expect("`current_folder` should not be null terminated")
                .into_bytes_with_nul()
        });
        self
    }

    /// Sets the absolute path of the file.
    #[must_use]
    pub fn current_file<P: AsRef<Path>>(mut self, current_file: impl Into<Option<P>>) -> Self {
        self.options.current_file = current_file.into().map(|c| {
            CString::new(c.as_ref().as_os_str().as_bytes())
                .expect("`current_folder` should not be null terminated")
                .into_bytes_with_nul()
        });
        self
    }

    /// Adds a files filter.
    #[must_use]
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.options.filters.push(filter);
        self
    }

    #[must_use]
    pub fn filters(mut self, filters: impl IntoIterator<Item = FileFilter>) -> Self {
        self.options.filters = filters.into_iter().collect();
        self
    }

    /// Sets the default filter.
    #[must_use]
    pub fn current_filter(mut self, current_filter: impl Into<Option<FileFilter>>) -> Self {
        self.options.current_filter = current_filter.into();
        self
    }

    /// Adds a choice.
    #[must_use]
    pub fn choice(mut self, choice: Choice) -> Self {
        self.options.choices.push(choice);
        self
    }

    #[must_use]
    pub fn choices(mut self, choices: impl IntoIterator<Item = Choice>) -> Self {
        self.options.choices = choices.into_iter().collect();
        self
    }

    pub async fn build(self) -> Result<Request<SelectedFiles>, Error> {
        let proxy = FileChooserProxy::new().await?;
        proxy
            .save_file(&self.identifier, &self.title, self.options)
            .await
    }
}
