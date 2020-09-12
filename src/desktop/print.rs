use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Print settings to set in the print dialog.
pub struct PrintSettings {
    /// One of landscape, portrait, reverse_landscape or reverse_portrait.
    pub orientation: Option<String>,
    /// A paper name according to [PWG 5101.1-2002](ftp://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn10-20020226-5101.1.pdf)
    pub paper_format: Option<String>,
    /// Paper width, in millimeters.
    pub paper_width: Option<String>,
    /// Paper height, in millimeters.
    pub paper_height: Option<String>,
    /// The number of copies to print.
    pub n_copies: Option<String>,
    /// The default paper source.
    pub default_source: Option<String>,
    /// Print quality, one of normal, high, low or draft.
    pub quality: Option<String>,
    /// The resolution, sets both resolution-x & resolution-y
    pub resolution: Option<String>,
    /// Whether to use color.
    pub use_color: Option<bool>,
    /// Duplex printing mode, one of simplex, horizontal or vertical.
    pub duplex: Option<String>,
    /// Whether to collate copies.
    pub collate: Option<bool>,
    /// Whether to reverse the order of printed pages.
    pub reverse: Option<bool>,
    /// A media type according to [PWG 5101.1-2002](ftp://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn10-20020226-5101.1.pdf)
    pub media_type: Option<String>,
    /// The dithering to use, one of fine, none, coarse, lineart, grayscale or error-diffusion.
    pub dither: Option<String>,
    /// The scale in percent
    pub scale: Option<String>,
    /// What pages to print, one of all, selection, current or ranges.
    pub print_pages: Option<String>,
    /// A list of page ranges, formatted like this: 0-2,4,9-11.
    pub page_ranges: Option<String>,
    /// What pages to print, one of all, even or odd.
    pub page_set: Option<String>,
    pub finishings: Option<String>,
    /// The number of pages per sheet.
    pub number_up: Option<String>,
    /// One of lrtb, lrbt, rltb, rlbt, tblr, tbrl, btlr, btrl.
    pub number_up_layout: Option<String>,
    pub output_bin: Option<String>,
    /// The horizontal resolution in dpi.
    pub resolution_x: Option<String>,
    /// The vertical resolution in dpi.
    pub resolution_y: Option<String>,
    /// The resolution in lpi (lines per inch).
    pub print_lpi: Option<String>,
    /// Basename to use for print-to-file.
    pub output_basename: Option<String>,
    /// Format to use for print-to-file, one of PDF, PS, SVG.
    pub output_file_format: Option<String>,
    /// The uri used for print-to file.
    pub output_uri: Option<String>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Setup the printed pages.
pub struct PrintPageSetup {
    /// the PPD name.
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
    /// Orientation, one of portrait, landscape, reverse-portrait or reverse-landscape.
    pub orientation: Option<f64>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a prepare print request.
pub struct PreparePrintOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// Whether to make the dialog modal.
    pub modal: bool,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a print request.
pub struct PrintOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// Whether to make the dialog modal.
    pub modal: bool,
    /// Token that was returned by a previous `prepare_print` call.
    pub token: String,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Print",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications print.
trait Print {
    /// Presents a print dialog to the user and returns print settings and page setup.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `title` - Title for the print dialog
    /// * `settings` - [`PrintSettings`]
    /// * `page_setup` - [`PrintPageSetup`]
    /// * `options` - [`PreparePrintOptions`]
    ///
    /// [`PrintSettings`]: ./struct.PrintSettings.html
    /// [`PrintPageSetup`]: ./struct.PrintPageSetup.html
    /// [`PreparePrintOptions`]: ./struct.PreparePrintOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn prepare_print(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        settings: PrintSettings,
        page_setup: PrintPageSetup,
        options: PreparePrintOptions,
    ) -> Result<OwnedObjectPath>;

    /// Asks to print a file.
    /// The file must be passed in the form of a file descriptor open for reading.
    /// This ensures that sandboxed applications only print files that they have access to.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - The application window identifier
    /// * `title` - The title for the print dialog
    /// * `fd` - File descriptor for reading the content to print
    /// * `options` - [`PrintOptions`]
    ///
    /// [`PrintOptions`]: ./struct.PrintOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn print(
        &self,
        parent_window: WindowIdentifier,
        title: &str,
        fd: Fd,
        options: PrintOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}
