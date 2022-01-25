//! # Examples
//!
//! Print a file
//!
//! ```rust,no_run
//! use ashpd::desktop::print::PrintProxy;
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = PrintProxy::new(&connection).await?;
//!     let identifier = WindowIdentifier::default();
//!
//!     let file = File::open("/home/bilelmoussaoui/gitlog.pdf").expect("file to print was not found");
//!     let pre_print = proxy
//!         .prepare_print(
//!             &identifier,
//!             "prepare print",
//!             Default::default(),
//!             Default::default(),
//!             true,
//!         )
//!         .await?;
//!     proxy
//!         .print(
//!             &identifier,
//!             "test",
//!             &file,
//!             Some(pre_print.token),
//!             true,
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize, Serializer};
use zbus::zvariant::{DeserializeDict, Fd, SerializeDict, Signature, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_request_method},
    Error, WindowIdentifier,
};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
/// The page orientation.
pub enum Orientation {
    /// Landscape.
    Landscape,
    /// Portrait.
    Portrait,
    /// Reverse landscape.
    ReverseLandscape,
    /// Reverse portrait.
    ReversePortrait,
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Landscape => write!(f, "Landscape"),
            Self::Portrait => write!(f, "Portrait"),
            Self::ReverseLandscape => write!(f, "Reverse Landscape"),
            Self::ReversePortrait => write!(f, "Reverse Portrait"),
        }
    }
}

impl AsRef<str> for Orientation {
    fn as_ref(&self) -> &str {
        match self {
            Self::Landscape => "Landscape",
            Self::Portrait => "Portrait",
            Self::ReverseLandscape => "Reverse Landscape",
            Self::ReversePortrait => "Reverse Portrait",
        }
    }
}

impl From<Orientation> for &'static str {
    fn from(o: Orientation) -> Self {
        match o {
            Orientation::Landscape => "Landscape",
            Orientation::Portrait => "Portrait",
            Orientation::ReverseLandscape => "Reverse Landscape",
            Orientation::ReversePortrait => "Reverse Portrait",
        }
    }
}

impl FromStr for Orientation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Landscape" | "landscape" => Ok(Orientation::Landscape),
            "Portrait" | "portrait" => Ok(Orientation::Portrait),
            "ReverseLandscape" | "reverse_landscape" => Ok(Orientation::ReverseLandscape),
            "ReversePortrait" | "reverse_portrait" => Ok(Orientation::ReversePortrait),
            _ => Err(Error::ParseError(
                "Failed to parse orientation, invalid value".to_string(),
            )),
        }
    }
}

impl Serialize for Orientation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Landscape => serializer.serialize_str("landscape"),
            Self::Portrait => serializer.serialize_str("portrait"),
            Self::ReverseLandscape => serializer.serialize_str("reverse_landscape"),
            Self::ReversePortrait => serializer.serialize_str("reverse_portrait"),
        }
    }
}

impl Type for Orientation {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
/// The print quality.
pub enum Quality {
    /// Draft quality.
    Draft,
    /// Low quality.
    Low,
    /// Normal quality.
    Normal,
    /// High quality.
    High,
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
        }
    }
}

impl AsRef<str> for Quality {
    fn as_ref(&self) -> &str {
        match self {
            Self::Draft => "Draft",
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
        }
    }
}

impl From<Quality> for &'static str {
    fn from(q: Quality) -> Self {
        match q {
            Quality::Draft => "Draft",
            Quality::Low => "Low",
            Quality::Normal => "Normal",
            Quality::High => "High",
        }
    }
}

impl FromStr for Quality {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" | "draft" => Ok(Quality::Draft),
            "Low" | "low" => Ok(Quality::Low),
            "Normal" | "normal" => Ok(Quality::Normal),
            "High" | "high" => Ok(Quality::High),
            _ => Err(Error::ParseError(
                "Failed to parse quality, invalid value".to_string(),
            )),
        }
    }
}

impl Serialize for Quality {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string().to_lowercase())
    }
}

impl Type for Quality {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Print settings to set in the print dialog.
#[zvariant(signature = "dict")]
pub struct Settings {
    /// One of landscape, portrait, reverse_landscape or reverse_portrait.
    pub orientation: Option<Orientation>,
    /// A paper name according to [PWG 5101.1-2002](ftp://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn10-20020226-5101.1.pdf)
    #[zvariant(rename = "paper-format")]
    pub paper_format: Option<String>,
    /// Paper width, in millimeters.
    #[zvariant(rename = "paper-width")]
    pub paper_width: Option<String>,
    /// Paper height, in millimeters.
    #[zvariant(rename = "paper-height")]
    pub paper_height: Option<String>,
    /// The number of copies to print.
    #[zvariant(rename = "n-copies")]
    pub n_copies: Option<String>,
    /// The default paper source.
    #[zvariant(rename = "default-source")]
    pub default_source: Option<String>,
    /// Print quality.
    pub quality: Option<Quality>,
    /// The resolution, sets both resolution-x & resolution-y
    pub resolution: Option<String>,
    /// Whether to use color.
    #[zvariant(rename = "use-color")]
    pub use_color: Option<bool>,
    /// Duplex printing mode, one of simplex, horizontal or vertical.
    pub duplex: Option<String>,
    /// Whether to collate copies.
    pub collate: Option<String>,
    /// Whether to reverse the order of printed pages.
    pub reverse: Option<String>,
    /// A media type according to [PWG 5101.1-2002](ftp://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn10-20020226-5101.1.pdf)
    #[zvariant(rename = "media-type")]
    pub media_type: Option<String>,
    /// The dithering to use, one of fine, none, coarse, lineart, grayscale or
    /// error-diffusion.
    pub dither: Option<String>,
    /// The scale in percent
    pub scale: Option<String>,
    /// What pages to print, one of all, selection, current or ranges.
    #[zvariant(rename = "print-pages")]
    pub print_pages: Option<String>,
    /// A list of page ranges, formatted like this: 0-2,4,9-11.
    #[zvariant(rename = "page-ranges")]
    pub page_ranges: Option<String>,
    /// What pages to print, one of all, even or odd.
    #[zvariant(rename = "page-set")]
    pub page_set: Option<String>,
    /// The finishings.
    pub finishings: Option<String>,
    /// The number of pages per sheet.
    #[zvariant(rename = "number-up")]
    pub number_up: Option<String>,
    /// One of lrtb, lrbt, rltb, rlbt, tblr, tbrl, btlr, btrl.
    #[zvariant(rename = "number-up-layout")]
    pub number_up_layout: Option<String>,
    #[zvariant(rename = "output-bin")]
    /// The output bin.
    pub output_bin: Option<String>,
    /// The horizontal resolution in dpi.
    #[zvariant(rename = "resolution-x")]
    pub resolution_x: Option<String>,
    /// The vertical resolution in dpi.
    #[zvariant(rename = "resolution-y")]
    pub resolution_y: Option<String>,
    /// The resolution in lpi (lines per inch).
    #[zvariant(rename = "printer-lpi")]
    pub print_lpi: Option<String>,
    /// Basename to use for print-to-file.
    #[zvariant(rename = "output-basename")]
    pub output_basename: Option<String>,
    /// Format to use for print-to-file, one of PDF, PS, SVG
    #[zvariant(rename = "output-file-format")]
    pub output_file_format: Option<String>,
    /// The uri used for print-to file.
    #[zvariant(rename = "output-uri")]
    pub output_uri: Option<String>,
}

impl Settings {
    /// Sets the orientation.
    #[must_use]
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Sets the paper name.
    #[must_use]
    pub fn paper_format(mut self, paper_format: &str) -> Self {
        self.paper_format = Some(paper_format.to_string());
        self
    }

    /// Sets the paper width.
    #[must_use]
    pub fn paper_width(mut self, paper_width: &str) -> Self {
        self.paper_width = Some(paper_width.to_string());
        self
    }

    /// Sets the paper height.
    #[must_use]
    pub fn paper_height(mut self, paper_height: &str) -> Self {
        self.paper_height = Some(paper_height.to_string());
        self
    }

    /// Sets the number of copies to print.
    #[must_use]
    pub fn n_copies(mut self, n_copies: &str) -> Self {
        self.n_copies = Some(n_copies.to_string());
        self
    }

    /// Sets the default paper source.
    #[must_use]
    pub fn default_source(mut self, default_source: &str) -> Self {
        self.default_source = Some(default_source.to_string());
        self
    }

    /// Sets the print quality.
    #[must_use]
    pub fn quality(mut self, quality: Quality) -> Self {
        self.quality = Some(quality);
        self
    }

    /// Sets the resolution, both resolution-x & resolution-y.
    #[must_use]
    pub fn resolution(mut self, resolution: &str) -> Self {
        self.resolution = Some(resolution.to_string());
        self
    }

    /// Sets whether to use color.
    #[must_use]
    pub fn use_color(mut self, use_color: bool) -> Self {
        self.use_color = Some(use_color);
        self
    }

    /// Sets the duplex printing mode.
    #[must_use]
    pub fn duplex(mut self, duplex: &str) -> Self {
        self.duplex = Some(duplex.to_string());
        self
    }

    /// Whether to collate copies.
    #[must_use]
    pub fn collate(mut self, collate: &str) -> Self {
        self.collate = Some(collate.to_string());
        self
    }

    /// Sets whether to reverse the order of the printed pages.
    #[must_use]
    pub fn reverse(mut self, reverse: &str) -> Self {
        self.reverse = Some(reverse.to_string());
        self
    }

    /// Sets the media type.
    #[must_use]
    pub fn media_type(mut self, media_type: &str) -> Self {
        self.media_type = Some(media_type.to_string());
        self
    }

    /// Sets the dithering to use.
    #[must_use]
    pub fn dither(mut self, dither: &str) -> Self {
        self.dither = Some(dither.to_string());
        self
    }

    /// Sets the page scale in percent.
    #[must_use]
    pub fn scale(mut self, scale: &str) -> Self {
        self.scale = Some(scale.to_string());
        self
    }

    /// Sets what pages to print, one of all, selection, current or ranges.
    #[must_use]
    pub fn print_pages(mut self, print_pages: &str) -> Self {
        self.print_pages = Some(print_pages.to_string());
        self
    }

    /// Sets a list of page ranges, formatted like this: 0-2,4,9-11.
    #[must_use]
    pub fn page_ranges(mut self, page_ranges: &str) -> Self {
        self.page_ranges = Some(page_ranges.to_string());
        self
    }

    /// Sets what pages to print, one of all, even or odd.
    #[must_use]
    pub fn page_set(mut self, page_set: &str) -> Self {
        self.page_set = Some(page_set.to_string());
        self
    }

    /// Sets the finishings.
    #[must_use]
    pub fn finishings(mut self, finishings: &str) -> Self {
        self.finishings = Some(finishings.to_string());
        self
    }

    /// Sets the number of pages per sheet.
    #[must_use]
    pub fn number_up(mut self, number_up: &str) -> Self {
        self.number_up = Some(number_up.to_string());
        self
    }

    /// Sets the number up layout, one of lrtb, lrbt, rltb, rlbt, tblr, tbrl,
    /// btlr, btrl.
    #[must_use]
    pub fn number_up_layout(mut self, number_up_layout: &str) -> Self {
        self.number_up_layout = Some(number_up_layout.to_string());
        self
    }

    /// Sets the output bin
    #[must_use]
    pub fn output_bin(mut self, output_bin: &str) -> Self {
        self.output_bin = Some(output_bin.to_string());
        self
    }

    /// Sets the horizontal resolution in dpi.
    #[must_use]
    pub fn resolution_x(mut self, resolution_x: &str) -> Self {
        self.resolution_x = Some(resolution_x.to_string());
        self
    }

    /// Sets the vertical resolution in dpi.
    #[must_use]
    pub fn resolution_y(mut self, resolution_y: &str) -> Self {
        self.resolution_y = Some(resolution_y.to_string());
        self
    }

    /// Sets the resolution in lines per inch.
    #[must_use]
    pub fn print_lpi(mut self, print_lpi: &str) -> Self {
        self.print_lpi = Some(print_lpi.to_string());
        self
    }

    /// Sets the print-to-file base name.
    #[must_use]
    pub fn output_basename(mut self, output_basename: &str) -> Self {
        self.output_basename = Some(output_basename.to_string());
        self
    }

    /// Sets the print-to-file format, one of PS, PDF, SVG.
    #[must_use]
    pub fn output_file_format(mut self, output_file_format: &str) -> Self {
        self.output_file_format = Some(output_file_format.to_string());
        self
    }

    /// Sets the print-to-file output uri.
    #[must_use]
    pub fn output_uri(mut self, output_uri: &str) -> Self {
        self.output_uri = Some(output_uri.to_string());
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Setup the printed pages.
#[zvariant(signature = "dict")]
pub struct PageSetup {
    /// the PPD name. It's the name to select a given driver.
    #[zvariant(rename = "PPDName")]
    pub ppdname: Option<String>,
    /// The name of the page setup.
    pub name: Option<String>,
    /// The user-visible name of the page setup.
    pub display_name: Option<String>,
    /// Paper width in millimeters.
    pub width: Option<f64>,
    /// Paper height in millimeters.
    pub height: Option<f64>,
    /// Top margin in millimeters.
    pub margin_top: Option<f64>,
    /// Bottom margin in millimeters.
    pub margin_bottom: Option<f64>,
    /// Right margin in millimeters.
    pub margin_right: Option<f64>,
    /// Left margin in millimeters.
    pub margin_left: Option<f64>,
    /// The page orientation.
    pub orientation: Option<Orientation>,
}

impl PageSetup {
    /// Sets the ppdname.
    #[must_use]
    pub fn ppdname(mut self, ppdname: &str) -> Self {
        self.ppdname = Some(ppdname.to_string());
        self
    }

    /// Sets the name of the page setup.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the user visible name of the page setup.
    #[must_use]
    pub fn display_name(mut self, display_name: &str) -> Self {
        self.display_name = Some(display_name.to_string());
        self
    }

    /// Sets the orientation.
    #[must_use]
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Sets the page width.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the page height.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the page top margin.
    #[must_use]
    pub fn margin_top(mut self, margin_top: f64) -> Self {
        self.margin_top = Some(margin_top);
        self
    }

    /// Sets the page bottom margin.
    #[must_use]
    pub fn margin_bottom(mut self, margin_bottom: f64) -> Self {
        self.margin_bottom = Some(margin_bottom);
        self
    }

    /// Sets the page right margin.
    #[must_use]
    pub fn margin_right(mut self, margin_right: f64) -> Self {
        self.margin_right = Some(margin_right);
        self
    }

    /// Sets the page margin left.
    #[must_use]
    pub fn margin_left(mut self, margin_left: f64) -> Self {
        self.margin_left = Some(margin_left);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`PrintProxy::prepare_print`] request.
#[zvariant(signature = "dict")]
struct PreparePrintOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether to make the dialog modal.
    modal: Option<bool>,
}

impl PreparePrintOptions {
    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`PrintProxy::print`] request.
#[zvariant(signature = "dict")]
struct PrintOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether to make the dialog modal.
    modal: Option<bool>,
    /// Token that was returned by a previous [`PrintProxy::prepare_print`]
    /// call.
    token: Option<u32>,
}

impl PrintOptions {
    /// A token retrieved from [`PrintProxy::prepare_print`].
    pub fn token(mut self, token: u32) -> Self {
        self.token = Some(token);
        self
    }

    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }
}

#[derive(DeserializeDict, SerializeDict, Type, Debug)]
/// A response to a [`PrintProxy::prepare_print`] request.
#[zvariant(signature = "dict")]
pub struct PreparePrint {
    /// The printing settings.
    pub settings: Settings,
    #[zvariant(rename = "page-setup")]
    /// The printed pages setup.
    pub page_setup: PageSetup,
    /// A token to pass to the print request.
    pub token: u32,
}

/// The interface lets sandboxed applications print.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Print`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Print).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Print")]
pub struct PrintProxy<'a>(zbus::Proxy<'a>);

impl<'a> PrintProxy<'a> {
    /// Create a new instance of [`PrintProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<PrintProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Print")?
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

    /// Presents a print dialog to the user and returns print settings and page
    /// setup.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `title` - Title for the print dialog.
    /// * `settings` - [`Settings`].
    /// * `page_setup` - [`PageSetup`].
    /// * `modal` - Whether the dialog should be a modal.
    ///
    /// # Specifications
    ///
    /// See also [`PreparePrint`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Print.PreparePrint).
    #[doc(alias = "PreparePrint")]
    #[doc(alias = "xdp_portal_prepare_print")]
    pub async fn prepare_print(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        settings: Settings,
        page_setup: PageSetup,
        modal: bool,
    ) -> Result<PreparePrint, Error> {
        let options = PreparePrintOptions::default().modal(modal);
        call_request_method(
            self.inner(),
            &options.handle_token,
            "PreparePrint",
            &(&identifier, title, settings, page_setup, &options),
        )
        .await
    }

    /// Asks to print a file.
    /// The file must be passed in the form of a file descriptor open for
    /// reading. This ensures that sandboxed applications only print files
    /// that they have access to.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The application window identifier.
    /// * `title` - The title for the print dialog.
    /// * `fd` - File descriptor for reading the content to print.
    /// * `token` - A token returned by a call to
    ///   [`prepare_print()`][`PrintProxy::prepare_print`].
    /// * `modal` - Whether the dialog should be a modal.
    ///
    /// # Specifications
    ///
    /// See also [`Print`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Print.Print).
    #[doc(alias = "Print")]
    #[doc(alias = "xdp_portal_print_file")]
    pub async fn print(
        &self,
        identifier: &WindowIdentifier,
        title: &str,
        fd: &impl AsRawFd,
        token: Option<u32>,
        modal: bool,
    ) -> Result<(), Error> {
        let options = PrintOptions::default()
            .token(token.unwrap_or(0))
            .modal(modal);
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "Print",
            &(&identifier, title, Fd::from(fd.as_raw_fd()), &options),
        )
        .await
    }
}
