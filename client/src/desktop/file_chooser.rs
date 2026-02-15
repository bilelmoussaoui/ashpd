//! The interface lets sandboxed applications ask the user for access to files
//! outside the sandbox. The portal backend will present the user with a file
//! chooser dialog.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.FileChooser`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html).
//!
//! ### Examples
//!
//! #### Opening a file
//!
//! ```rust,no_run
//! use ashpd::desktop::file_chooser::{Choice, FileFilter, SelectedFiles};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let files = SelectedFiles::open_file()
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
//!         .send()
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
//! use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let files = SelectedFiles::save_file()
//!         .title("open a file to write")
//!         .accept_label("write")
//!         .current_name("image.jpg")
//!         .modal(true)
//!         .filter(FileFilter::new("JPEG Image").glob("*.jpg"))
//!         .send()
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
//! use ashpd::desktop::file_chooser::SelectedFiles;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let files = SelectedFiles::save_files()
//!         .title("open files to write")
//!         .accept_label("write files")
//!         .modal(true)
//!         .current_folder("/home/bilelmoussaoui/Pictures")?
//!         .files(&["test.jpg", "awesome.png"])?
//!         .send()
//!         .await?
//!         .response()?;
//!
//!     println!("{:#?}", files);
//!
//!     Ok(())
//! }
//! ```

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{
    Optional, Type,
    as_value::{self, optional},
};

use super::{HandleToken, Request};
use crate::{Error, FilePath, Uri, WindowIdentifier, proxy::Proxy};

#[derive(Clone, Serialize, Deserialize, Type, Debug, PartialEq)]
/// A file filter, to limit the available file choices to a mimetype or a glob
/// pattern.
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Clone, Serialize_repr, Deserialize_repr, Debug, Type, PartialEq)]
#[repr(u32)]
enum FilterType {
    GlobPattern = 0,
    MimeType = 1,
}

impl FilterType {
    /// Whether it is a mime type filter.
    fn is_mimetype(&self) -> bool {
        matches!(self, FilterType::MimeType)
    }

    /// Whether it is a glob pattern type filter.
    fn is_pattern(&self) -> bool {
        matches!(self, FilterType::GlobPattern)
    }
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

impl FileFilter {
    /// The label of the filter.
    pub fn label(&self) -> &str {
        &self.0
    }

    /// List of mimetypes filters.
    pub fn mimetype_filters(&self) -> Vec<&str> {
        self.1
            .iter()
            .filter_map(|(type_, string)| type_.is_mimetype().then_some(string.as_str()))
            .collect()
    }

    /// List of glob patterns filters.
    pub fn pattern_filters(&self) -> Vec<&str> {
        self.1
            .iter()
            .filter_map(|(type_, string)| type_.is_pattern().then_some(string.as_str()))
            .collect()
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

    /// Pairs of choices.
    pub fn pairs(&self) -> Vec<(&str, &str)> {
        self.2
            .iter()
            .map(|(x, y)| (x.as_str(), y.as_str()))
            .collect::<Vec<_>>()
    }

    /// The initially selected value.
    pub fn initial_selection(&self) -> &str {
        &self.3
    }
}

#[derive(Serialize, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct OpenFileOptions {
    #[serde(with = "as_value")]
    handle_token: HandleToken,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    accept_label: Option<String>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    modal: Option<bool>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    multiple: Option<bool>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    directory: Option<bool>,
    #[serde(with = "as_value", skip_serializing_if = "Vec::is_empty")]
    filters: Vec<FileFilter>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_filter: Option<FileFilter>,
    #[serde(with = "as_value", skip_serializing_if = "Vec::is_empty")]
    choices: Vec<Choice>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_folder: Option<FilePath>,
}

#[derive(Serialize, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct SaveFileOptions {
    #[serde(with = "as_value")]
    handle_token: HandleToken,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    accept_label: Option<String>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    modal: Option<bool>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_name: Option<String>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_folder: Option<FilePath>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_file: Option<FilePath>,
    #[serde(with = "as_value", skip_serializing_if = "Vec::is_empty")]
    filters: Vec<FileFilter>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_filter: Option<FileFilter>,
    #[serde(with = "as_value", skip_serializing_if = "Vec::is_empty")]
    choices: Vec<Choice>,
}

#[derive(Serialize, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct SaveFilesOptions {
    #[serde(with = "as_value")]
    handle_token: HandleToken,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    accept_label: Option<String>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    modal: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    choices: Vec<Choice>,
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    current_folder: Option<FilePath>,
    #[serde(with = "as_value", skip_serializing_if = "Vec::is_empty")]
    files: Vec<FilePath>,
}

#[derive(Deserialize, Type, Debug)]
/// A response of [`OpenFileRequest`], [`SaveFileRequest`] or
/// [`SaveFilesRequest`].
#[zvariant(signature = "dict")]
pub struct SelectedFiles {
    #[serde(default, with = "as_value")]
    uris: Vec<Uri>,
    #[serde(default, with = "as_value")]
    choices: Vec<(String, String)>,
}

impl SelectedFiles {
    /// Start an open file request.
    pub fn open_file() -> OpenFileRequest {
        OpenFileRequest::default()
    }

    /// Start a save file request.
    pub fn save_file() -> SaveFileRequest {
        SaveFileRequest::default()
    }

    /// Start a save files request.
    pub fn save_files() -> SaveFilesRequest {
        SaveFilesRequest::default()
    }

    /// The selected files uris.
    pub fn uris(&self) -> &[Uri] {
        self.uris.as_slice()
    }

    /// The selected value of each choice as a tuple of (key, value)
    pub fn choices(&self) -> &[(String, String)] {
        &self.choices
    }
}

#[doc(alias = "org.freedesktop.portal.FileChooser")]
struct FileChooserProxy(Proxy<'static>);

impl FileChooserProxy {
    /// Create a new instance of [`FileChooserProxy`].
    pub async fn new() -> Result<Self, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.FileChooser").await?;
        Ok(Self(proxy))
    }

    pub async fn with_connection(connection: zbus::Connection) -> Result<Self, Error> {
        let proxy =
            Proxy::new_desktop_with_connection(connection, "org.freedesktop.portal.FileChooser")
                .await?;
        Ok(Self(proxy))
    }

    #[doc(alias = "OpenFile")]
    pub async fn open_file(
        &self,
        identifier: Option<&WindowIdentifier>,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<Request<SelectedFiles>, Error> {
        let identifier = Optional::from(identifier);
        self.0
            .request(
                &options.handle_token,
                "OpenFile",
                &(identifier, title, &options),
            )
            .await
    }

    #[doc(alias = "SaveFile")]
    pub async fn save_file(
        &self,
        identifier: Option<&WindowIdentifier>,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<Request<SelectedFiles>, Error> {
        let identifier = Optional::from(identifier);
        self.0
            .request(
                &options.handle_token,
                "SaveFile",
                &(identifier, title, &options),
            )
            .await
    }

    #[doc(alias = "SaveFiles")]
    pub async fn save_files(
        &self,
        identifier: Option<&WindowIdentifier>,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<Request<SelectedFiles>, Error> {
        let identifier = Optional::from(identifier);
        self.0
            .request(
                &options.handle_token,
                "SaveFiles",
                &(identifier, title, &options),
            )
            .await
    }
}

impl std::ops::Deref for FileChooserProxy {
    type Target = zbus::Proxy<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_open_file")]
/// A [builder-pattern] type to open a file.
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct OpenFileRequest {
    identifier: Option<WindowIdentifier>,
    title: String,
    options: OpenFileOptions,
    connection: Option<zbus::Connection>,
}

impl OpenFileRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into();
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
    /// Adds a list of files filters.
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
    /// Adds a list of choices.
    pub fn choices(mut self, choices: impl IntoIterator<Item = Choice>) -> Self {
        self.options.choices = choices.into_iter().collect();
        self
    }

    /// Specifies the current folder path.
    pub fn current_folder<P: AsRef<Path>>(
        mut self,
        current_folder: impl Into<Option<P>>,
    ) -> Result<Self, crate::Error> {
        self.options.current_folder = current_folder
            .into()
            .map(|c| FilePath::new(c))
            .transpose()?;
        Ok(self)
    }

    #[must_use]
    /// Sets a connection to use other than the internal one.
    pub fn connection(mut self, connection: Option<zbus::Connection>) -> Self {
        self.connection = connection;
        self
    }

    /// Send the request.
    pub async fn send(self) -> Result<Request<SelectedFiles>, Error> {
        let proxy = if let Some(connection) = self.connection {
            FileChooserProxy::with_connection(connection).await?
        } else {
            FileChooserProxy::new().await?
        };
        proxy
            .open_file(self.identifier.as_ref(), &self.title, self.options)
            .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_save_files")]
/// A [builder-pattern] type to save multiple files.
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct SaveFilesRequest {
    identifier: Option<WindowIdentifier>,
    title: String,
    options: SaveFilesOptions,
    connection: Option<zbus::Connection>,
}

impl SaveFilesRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into();
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
    /// Adds a list of choices.
    pub fn choices(mut self, choices: impl IntoIterator<Item = Choice>) -> Self {
        self.options.choices = choices.into_iter().collect();
        self
    }

    /// Specifies the current folder path.
    pub fn current_folder<P: AsRef<Path>>(
        mut self,
        current_folder: impl Into<Option<P>>,
    ) -> Result<Self, crate::Error> {
        self.options.current_folder = current_folder
            .into()
            .map(|c| FilePath::new(c))
            .transpose()?;
        Ok(self)
    }

    /// Sets a list of files to save.
    pub fn files<P: IntoIterator<Item = impl AsRef<Path>>>(
        mut self,
        files: impl Into<Option<P>>,
    ) -> Result<Self, crate::Error> {
        if let Some(f) = files.into() {
            self.options.files = f
                .into_iter()
                .map(|s| FilePath::new(s))
                .collect::<Result<Vec<_>, _>>()?;
        }
        Ok(self)
    }

    #[must_use]
    /// Sets a connection to use other than the internal one.
    pub fn connection(mut self, connection: Option<zbus::Connection>) -> Self {
        self.connection = connection;
        self
    }

    /// Send the request.
    pub async fn send(self) -> Result<Request<SelectedFiles>, Error> {
        let proxy = if let Some(connection) = self.connection {
            FileChooserProxy::with_connection(connection).await?
        } else {
            FileChooserProxy::new().await?
        };
        proxy
            .save_files(self.identifier.as_ref(), &self.title, self.options)
            .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_save_file")]
/// A [builder-pattern] type to save a file.
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct SaveFileRequest {
    identifier: Option<WindowIdentifier>,
    title: String,
    options: SaveFileOptions,
    connection: Option<zbus::Connection>,
}

impl SaveFileRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into();
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
    pub fn current_folder<P: AsRef<Path>>(
        mut self,
        current_folder: impl Into<Option<P>>,
    ) -> Result<Self, crate::Error> {
        self.options.current_folder = current_folder
            .into()
            .map(|c| FilePath::new(c))
            .transpose()?;
        Ok(self)
    }

    /// Sets the absolute path of the file.
    pub fn current_file<P: AsRef<Path>>(
        mut self,
        current_file: impl Into<Option<P>>,
    ) -> Result<Self, crate::Error> {
        self.options.current_file = current_file.into().map(|c| FilePath::new(c)).transpose()?;
        Ok(self)
    }

    /// Adds a files filter.
    #[must_use]
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.options.filters.push(filter);
        self
    }

    #[must_use]
    /// Adds a list of files filters.
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
    /// Adds a list of choices.
    pub fn choices(mut self, choices: impl IntoIterator<Item = Choice>) -> Self {
        self.options.choices = choices.into_iter().collect();
        self
    }

    #[must_use]
    /// Sets a connection to use other than the internal one.
    pub fn connection(mut self, connection: Option<zbus::Connection>) -> Self {
        self.connection = connection;
        self
    }

    /// Send the request.
    pub async fn send(self) -> Result<Request<SelectedFiles>, Error> {
        let proxy = if let Some(connection) = self.connection {
            FileChooserProxy::with_connection(connection).await?
        } else {
            FileChooserProxy::new().await?
        };
        proxy
            .save_file(self.identifier.as_ref(), &self.title, self.options)
            .await
    }
}
