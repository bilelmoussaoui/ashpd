use adw::prelude::*;
use ashpd::{
    desktop::file_chooser::{OpenFileRequest, SaveFileRequest, SaveFilesRequest},
    WindowIdentifier,
};
use glib::clone;
use gtk::{glib, subclass::prelude::*};

use crate::{
    portals::{is_empty, split_comma},
    widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use adw::subclass::prelude::*;

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
            Self::bind_template(klass);

            klass.install_action(
                "file_chooser.open_file",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.open_file().await;
                    }));
                },
            );
            klass.install_action(
                "file_chooser.save_file",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.save_file().await;
                    }));
                },
            );
            klass.install_action(
                "file_chooser.save_files",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.save_files().await;
                    }));
                },
            );
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
    pub struct FileChooserPage(ObjectSubclass<imp::FileChooserPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl FileChooserPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a FileChooserPage")
    }

    async fn open_file(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.open_title_entry.text();
        let directory = imp.open_directory_switch.is_active();
        let modal = imp.open_modal_switch.is_active();
        let multiple = imp.open_multiple_switch.is_active();

        let mut request = OpenFileRequest::default()
            .directory(directory)
            .identifier(identifier)
            .modal(modal)
            .title(&title)
            .multiple(multiple);
        if let Some(accept_label) = is_empty(imp.open_accept_label_entry.text()) {
            request.set_accept_label(&accept_label)
        }
        match request.build().await {
            Ok(files) => {
                imp.open_response_group.show();

                for uri in files.uris() {
                    imp.open_response_group
                        .add(&adw::ActionRow::builder().title(uri.as_str()).build());
                }
                self.send_notification(
                    "Open file request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => {
                self.send_notification("Request to open a file failed", NotificationKind::Error);
            }
        }
    }

    async fn save_file(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.save_file_title_entry.text();
        let modal = imp.save_file_modal_switch.is_active();
        let mut request = SaveFileRequest::default()
            .identifier(identifier)
            .modal(modal)
            .title(&title);

        if let Some(accept_label) = is_empty(imp.save_file_accept_label_entry.text()) {
            request.set_accept_label(&accept_label);
        }
        if let Some(current_name) = is_empty(imp.save_file_current_name_entry.text()) {
            request.set_current_name(&current_name);
        }
        if let Some(current_folder) = is_empty(imp.save_file_current_folder_entry.text()) {
            request.set_current_folder(current_folder);
        }
        if let Some(current_file) = is_empty(imp.save_file_current_file_entry.text()) {
            request.set_current_file(current_file);
        }
        match request.build().await {
            Ok(files) => {
                imp.save_file_response_group.show();

                for uri in files.uris() {
                    imp.save_file_response_group
                        .add(&adw::ActionRow::builder().title(uri.as_str()).build());
                }

                self.send_notification(
                    "Save file request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => {
                self.send_notification("Request to save a file failed", NotificationKind::Error);
            }
        }
    }

    async fn save_files(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.save_files_title_entry.text();
        let modal = imp.save_files_modal_switch.is_active();
        let mut request = SaveFilesRequest::default()
            .identifier(identifier)
            .modal(modal)
            .title(&title);

        if let Some(accept_label) = is_empty(imp.save_files_accept_label_entry.text()) {
            request.set_accept_label(&accept_label);
        }
        if let Some(current_folder) = is_empty(imp.save_files_current_folder_entry.text()) {
            request.set_current_folder(current_folder);
        }

        if let Some(files) = is_empty(imp.save_files_files_entry.text()).map(split_comma) {
            request.set_files(
                files
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            );
        };
        match request.build().await {
            Ok(files) => {
                imp.save_files_response_group.show();

                for uri in files.uris() {
                    imp.save_files_response_group
                        .add(&adw::ActionRow::builder().title(uri.as_str()).build());
                }
                self.send_notification(
                    "Save files request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => {
                self.send_notification("Request to save files failed", NotificationKind::Error);
            }
        }
    }
}
