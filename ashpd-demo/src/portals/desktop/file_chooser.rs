use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    desktop::file_chooser::{OpenFileRequest, SaveFileRequest, SaveFilesRequest},
    WindowIdentifier,
};
use gtk::glib;

use crate::{
    portals::{is_empty, split_comma},
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/file_chooser.ui")]
    pub struct FileChooserPage {
        #[template_child]
        pub open_title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub open_accept_label_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub open_modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub open_multiple_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub open_directory_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub open_response_group: TemplateChild<adw::PreferencesGroup>,

        #[template_child]
        pub save_file_title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_file_current_file_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_file_current_name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_file_accept_label_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_file_current_folder_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_file_modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub save_file_response_group: TemplateChild<adw::PreferencesGroup>,

        #[template_child]
        pub save_files_title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_accept_label_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_current_folder_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub save_files_files_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileChooserPage {
        const NAME: &'static str = "FileChooserPage";
        type Type = super::FileChooserPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("file_chooser.open_file", None, |page, _, _| async move {
                page.open_file().await;
            });
            klass.install_action_async("file_chooser.save_file", None, |page, _, _| async move {
                if let Err(err) = page.save_file().await {
                    tracing::error!("Failed to pick a file {err}");
                }
            });
            klass.install_action_async("file_chooser.save_files", None, |page, _, _| async move {
                if let Err(err) = page.save_files().await {
                    tracing::error!("Failed to pick files {err}");
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for FileChooserPage {}
    impl WidgetImpl for FileChooserPage {}
    impl BinImpl for FileChooserPage {}
    impl PortalPageImpl for FileChooserPage {}
}

glib::wrapper! {
    pub struct FileChooserPage(ObjectSubclass<imp::FileChooserPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl FileChooserPage {
    async fn open_file(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.open_title_entry.text();
        let directory = imp.open_directory_switch.is_active();
        let modal = imp.open_modal_switch.is_active();
        let multiple = imp.open_multiple_switch.is_active();
        let accept_label = is_empty(imp.open_accept_label_entry.text());

        let request = OpenFileRequest::default()
            .directory(directory)
            .identifier(identifier)
            .modal(modal)
            .title(&*title)
            .multiple(multiple)
            .accept_label(accept_label.as_deref());
        match request.send().await.and_then(|r| r.response()) {
            Ok(files) => {
                imp.open_response_group.set_visible(true);

                for uri in files.uris() {
                    imp.open_response_group.add(
                        &adw::ActionRow::builder()
                            .title(uri.as_str())
                            .title_selectable(true)
                            .build(),
                    );
                }
                self.success("Open file request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to open a file: {err}");
                self.error("Request to open a file failed");
            }
        }
    }

    async fn save_file(&self) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.save_file_title_entry.text();
        let modal = imp.save_file_modal_switch.is_active();
        let accept_label = is_empty(imp.save_file_accept_label_entry.text());
        let current_name = is_empty(imp.save_file_current_name_entry.text());
        let current_folder = is_empty(imp.save_file_current_folder_entry.text());
        let current_file = is_empty(imp.save_file_current_file_entry.text());
        let request = SaveFileRequest::default()
            .identifier(identifier)
            .modal(modal)
            .title(&*title)
            .accept_label(accept_label.as_deref())
            .current_name(current_name.as_deref())
            .current_folder::<String>(current_folder)?
            .current_file::<String>(current_file)?;
        match request.send().await.and_then(|r| r.response()) {
            Ok(files) => {
                imp.save_file_response_group.set_visible(true);

                for uri in files.uris() {
                    imp.save_file_response_group.add(
                        &adw::ActionRow::builder()
                            .title(uri.as_str())
                            .title_selectable(true)
                            .build(),
                    );
                }

                self.success("Save file request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to save a file: {err}");
                self.error("Request to save a file failed");
            }
        }
        Ok(())
    }

    async fn save_files(&self) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.save_files_title_entry.text();
        let modal = imp.save_files_modal_switch.is_active();
        let accept_label = is_empty(imp.save_files_accept_label_entry.text());
        let current_folder = is_empty(imp.save_files_current_folder_entry.text());
        let files = is_empty(imp.save_files_files_entry.text()).map(split_comma);
        let request = SaveFilesRequest::default()
            .identifier(identifier)
            .modal(modal)
            .title(&*title)
            .accept_label(accept_label.as_deref())
            .current_folder::<String>(current_folder)?
            .files::<Vec<_>>(files)?;

        match request.send().await.and_then(|r| r.response()) {
            Ok(files) => {
                imp.save_files_response_group.set_visible(true);

                for uri in files.uris() {
                    imp.save_files_response_group.add(
                        &adw::ActionRow::builder()
                            .title(uri.as_str())
                            .title_selectable(true)
                            .build(),
                    );
                }
                self.success("Save files request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to save files: {err}");
                self.error("Request to save files failed");
            }
        }
        Ok(())
    }
}
