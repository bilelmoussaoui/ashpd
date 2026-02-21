use std::os::fd::AsFd;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    WindowIdentifier,
    desktop::print::{
        Dither, Duplex, NumberUpLayout, Orientation, OutputFileFormat, PageSet, PageSetup,
        PreparePrintOptions, PrintOptions, PrintPages, PrintProxy, Quality, Settings,
    },
};
use glib::translate::*;
use gtk::{gio, glib};

use crate::{
    portals::{is_empty, spawn_tokio},
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/print.ui")]
    pub struct PrintPage {
        #[template_child]
        pub title: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub modal_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub accept_label: TemplateChild<adw::EntryRow>,
        // Settings options
        #[template_child]
        pub paper_format: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub paper_width: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub paper_height: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub n_copies_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub default_source: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub orientation_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub quality_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub resolution: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub resolution_x_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub resolution_y_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub duplex_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub use_color_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub collate_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub reverse_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub media_type: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub dither_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub scale_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub print_pages_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub page_ranges: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub page_set_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub finishings: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub number_up_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub number_up_layout_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub output_bin: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub printer_lpi: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub output_basename: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub output_file_format_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub output_uri: TemplateChild<adw::EntryRow>,
        // PageSetup options
        #[template_child]
        pub ppdname: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub page_setup_name: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub display_name: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub width_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub height_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub margin_top_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub margin_bottom_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub margin_left_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub margin_right_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub page_setup_orientation_combo: TemplateChild<adw::ComboRow>,
        // PreparePrintOptions
        #[template_child]
        pub has_current_page_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub has_selected_pages_switch: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PrintPage {
        const NAME: &'static str = "PrintPage";
        type Type = super::PrintPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Dither::ensure_type();
            Duplex::ensure_type();
            NumberUpLayout::ensure_type();
            Orientation::ensure_type();
            OutputFileFormat::ensure_type();
            PageSet::ensure_type();
            Quality::ensure_type();
            PrintPages::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action_async("print.select_file", None, |page, _, _| async move {
                if let Err(err) = page.select_file().await {
                    tracing::error!("Failed to pick a file {err}");
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl PrintPage {
        #[template_callback]
        fn orientation_name(item: &adw::EnumListItem) -> String {
            let orientation = unsafe { Orientation::from_glib(item.value()) };
            match orientation {
                Orientation::Landscape => String::from("Landscape"),
                Orientation::Portrait => String::from("Portrait"),
                Orientation::ReverseLandscape => String::from("Reverse Landscape"),
                Orientation::ReversePortrait => String::from("Reverse Portrait"),
            }
        }

        #[template_callback]
        fn quality_name(item: &adw::EnumListItem) -> String {
            let quality = unsafe { Quality::from_glib(item.value()) };
            match quality {
                Quality::Draft => String::from("Draft"),
                Quality::Low => String::from("Low"),
                Quality::Normal => String::from("Normal"),
                Quality::High => String::from("High"),
            }
        }

        #[template_callback]
        fn duplex_name(item: &adw::EnumListItem) -> String {
            let duplex = unsafe { Duplex::from_glib(item.value()) };
            match duplex {
                Duplex::Simplex => String::from("Simplex"),
                Duplex::Horizontal => String::from("Horizontal"),
                Duplex::Vertical => String::from("Vertical"),
            }
        }

        #[template_callback]
        fn dither_name(item: &adw::EnumListItem) -> String {
            let dither = unsafe { Dither::from_glib(item.value()) };
            match dither {
                Dither::Fine => String::from("Fine"),
                Dither::None => String::from("None"),
                Dither::Coarse => String::from("Coarse"),
                Dither::Lineart => String::from("Line Art"),
                Dither::Grayscale => String::from("Grayscale"),
                Dither::ErrorDiffusion => String::from("Error Diffusion"),
            }
        }

        #[template_callback]
        fn print_pages_name(item: &adw::EnumListItem) -> String {
            let print_pages = unsafe { PrintPages::from_glib(item.value()) };
            match print_pages {
                PrintPages::All => String::from("All"),
                PrintPages::Selection => String::from("Selection"),
                PrintPages::Current => String::from("Current"),
                PrintPages::Ranges => String::from("Ranges"),
            }
        }

        #[template_callback]
        fn page_set_name(item: &adw::EnumListItem) -> String {
            let page_set = unsafe { PageSet::from_glib(item.value()) };
            match page_set {
                PageSet::All => String::from("All"),
                PageSet::Even => String::from("Even"),
                PageSet::Odd => String::from("Odd"),
            }
        }

        #[template_callback]
        fn number_up_layout_name(item: &adw::EnumListItem) -> String {
            let layout = unsafe { NumberUpLayout::from_glib(item.value()) };
            match layout {
                NumberUpLayout::Lrtb => String::from("Left to Right, Top to Bottom"),
                NumberUpLayout::Lrbt => String::from("Left to Right, Bottom to Top"),
                NumberUpLayout::Rltb => String::from("Right to Left, Top to Bottom"),
                NumberUpLayout::Rlbt => String::from("Right to Left, Bottom to Top"),
                NumberUpLayout::Tblr => String::from("Top to Bottom, Left to Right"),
                NumberUpLayout::Tbrl => String::from("Top to Bottom, Right to Left"),
                NumberUpLayout::Btlr => String::from("Bottom to Top, Left to Right"),
                NumberUpLayout::Btrl => String::from("Bottom to Top, Right to Left"),
            }
        }

        #[template_callback]
        fn output_file_format_name(item: &adw::EnumListItem) -> String {
            let format = unsafe { OutputFileFormat::from_glib(item.value()) };
            match format {
                OutputFileFormat::Pdf => String::from("PDF"),
                OutputFileFormat::Ps => String::from("PostScript"),
                OutputFileFormat::Svg => String::from("SVG"),
            }
        }
    }

    impl ObjectImpl for PrintPage {}
    impl WidgetImpl for PrintPage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { PrintProxy::new().await }).await {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for PrintPage {}
    impl PortalPageImpl for PrintPage {}
}

glib::wrapper! {
    pub struct PrintPage(ObjectSubclass<imp::PrintPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl PrintPage {
    async fn select_file(&self) -> anyhow::Result<()> {
        let imp = self.imp();
        let title = imp.title.text();
        let modal = imp.modal_switch.is_active();
        let accept_label = is_empty(imp.accept_label.text());
        let root = self.native().unwrap();

        // Collect Settings values
        let orientation = imp
            .orientation_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { Orientation::from_glib(item.value()) });
        let quality = imp
            .quality_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { Quality::from_glib(item.value()) });
        let duplex = imp
            .duplex_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { Duplex::from_glib(item.value()) });
        let use_color = imp.use_color_switch.is_active();
        let collate = imp.collate_switch.is_active();
        let reverse = imp.reverse_switch.is_active();
        let dither = imp
            .dither_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { Dither::from_glib(item.value()) });
        let scale = imp.scale_spin.value() as u32;
        let print_pages = imp
            .print_pages_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { PrintPages::from_glib(item.value()) });
        let page_set = imp
            .page_set_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { PageSet::from_glib(item.value()) });
        let number_up = imp.number_up_spin.value() as u32;
        let number_up_layout = imp
            .number_up_layout_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { NumberUpLayout::from_glib(item.value()) });
        let output_file_format = imp
            .output_file_format_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { OutputFileFormat::from_glib(item.value()) });

        // Collect string Settings values
        let paper_format = is_empty(imp.paper_format.text());
        let paper_width = is_empty(imp.paper_width.text());
        let paper_height = is_empty(imp.paper_height.text());
        let n_copies = imp.n_copies_spin.value() as u32;
        let default_source = is_empty(imp.default_source.text());
        let resolution = is_empty(imp.resolution.text());
        let resolution_x = imp.resolution_x_spin.value() as i32;
        let resolution_y = imp.resolution_y_spin.value() as i32;
        let media_type = is_empty(imp.media_type.text());
        let page_ranges = is_empty(imp.page_ranges.text());
        let finishings = is_empty(imp.finishings.text());
        let output_bin = is_empty(imp.output_bin.text());
        let printer_lpi = is_empty(imp.printer_lpi.text());
        let output_basename = is_empty(imp.output_basename.text());
        let output_uri = is_empty(imp.output_uri.text()).and_then(|s| s.parse().ok());

        // PageSetup values
        let ppdname = is_empty(imp.ppdname.text());
        let page_setup_name = is_empty(imp.page_setup_name.text());
        let display_name = is_empty(imp.display_name.text());
        let width = imp.width_spin.value();
        let height = imp.height_spin.value();
        let margin_top = imp.margin_top_spin.value();
        let margin_bottom = imp.margin_bottom_spin.value();
        let margin_left = imp.margin_left_spin.value();
        let margin_right = imp.margin_right_spin.value();
        let page_setup_orientation = imp
            .page_setup_orientation_combo
            .selected_item()
            .and_downcast::<adw::EnumListItem>()
            .map(|item| unsafe { Orientation::from_glib(item.value()) });

        // PreparePrintOptions values
        let has_current_page = imp.has_current_page_switch.is_active();
        let has_selected_pages = imp.has_selected_pages_switch.is_active();

        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("images"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .accept_label("Select")
            .modal(true)
            .filters(&filters)
            .build();

        let path = dialog
            .open_future(root.downcast_ref::<gtk::Window>())
            .await?
            .path()
            .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let settings = Settings::default()
            .set_paper_format(paper_format.as_deref())
            .set_paper_width(paper_width.as_deref())
            .set_paper_height(paper_height.as_deref())
            .set_n_copies(if n_copies > 0 { Some(n_copies) } else { None })
            .set_default_source(default_source.as_deref())
            .set_orientation(orientation)
            .set_quality(quality)
            .set_resolution(resolution.as_deref())
            .set_resolution_x(if resolution_x > 0 {
                Some(resolution_x)
            } else {
                None
            })
            .set_resolution_y(if resolution_y > 0 {
                Some(resolution_y)
            } else {
                None
            })
            .set_duplex(duplex)
            .set_use_color(use_color)
            .set_collate(collate)
            .set_reverse(reverse)
            .set_media_type(media_type.as_deref())
            .set_dither(dither)
            .set_scale(if scale > 0 { Some(scale) } else { None })
            .set_print_pages(print_pages)
            .set_page_ranges(page_ranges.as_deref())
            .set_page_set(page_set)
            .set_finishings(finishings.as_deref())
            .set_number_up(if number_up > 0 { Some(number_up) } else { None })
            .set_number_up_layout(number_up_layout)
            .set_output_bin(output_bin.as_deref())
            .set_printer_lpi(printer_lpi.as_deref())
            .set_output_basename(output_basename.as_deref())
            .set_output_file_format(output_file_format)
            .set_output_uri(output_uri.as_ref());

        let page_setup = PageSetup::default()
            .set_ppdname(ppdname.as_deref())
            .set_name(page_setup_name.as_deref())
            .set_display_name(display_name.as_deref())
            .set_width(if width > 0.0 { Some(width) } else { None })
            .set_height(if height > 0.0 { Some(height) } else { None })
            .set_margin_top(if margin_top > 0.0 {
                Some(margin_top)
            } else {
                None
            })
            .set_margin_bottom(if margin_bottom > 0.0 {
                Some(margin_bottom)
            } else {
                None
            })
            .set_margin_left(if margin_left > 0.0 {
                Some(margin_left)
            } else {
                None
            })
            .set_margin_right(if margin_right > 0.0 {
                Some(margin_right)
            } else {
                None
            })
            .set_orientation(page_setup_orientation);

        let prepare_options = PreparePrintOptions::default()
            .set_modal(modal)
            .set_accept_label(accept_label.as_deref())
            .set_has_current_page(has_current_page)
            .set_has_selected_pages(has_selected_pages)
            .set_supported_output_file_formats(
                output_file_format.map(|f| vec![f]).unwrap_or_default(),
            );

        let print_options = PrintOptions::default()
            .set_modal(modal)
            .set_supported_output_file_formats(
                output_file_format.map(|f| vec![f]).unwrap_or_default(),
            );

        match print(
            identifier,
            &title,
            file,
            settings,
            page_setup,
            prepare_options,
            print_options,
        )
        .await
        {
            Ok(_) => {
                self.success("Print request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to print {}", err);
                self.error(&format!("Request to print failed: {err}"));
            }
        }
        Ok(())
    }
}

async fn print(
    identifier: Option<WindowIdentifier>,
    title: &str,
    file: std::fs::File,
    settings: Settings,
    page_setup: PageSetup,
    prepare_options: PreparePrintOptions,
    print_options: PrintOptions,
) -> ashpd::Result<()> {
    let owned_title = title.to_owned();
    spawn_tokio(async move {
        let proxy = PrintProxy::new().await?;

        let out = proxy
            .prepare_print(
                identifier.as_ref(),
                &owned_title,
                settings,
                page_setup,
                prepare_options,
            )
            .await?
            .response()?;

        proxy
            .print(
                identifier.as_ref(),
                &owned_title,
                &file.as_fd(),
                print_options.set_token(out.token),
            )
            .await
    })
    .await?;
    Ok(())
}
