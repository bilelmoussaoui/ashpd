use crate::portals::{is_empty, split_comma};
use ashpd::{
    desktop::file_chooser::{
        FileChooserProxy, OpenFileOptions, SaveFileOptions, SaveFilesOptions, SelectedFiles,
    },
    zbus, WindowIdentifier,
};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk_macros::spawn;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/file_chooser.ui")]
    pub struct FileChooserPage {
        #[template_child]
        pub open_title_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub open_accept_label_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub open_modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub open_multiple_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub open_directory_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub open_response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub open_uris_listbox: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub save_file_title_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_file_current_file_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_file_current_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_file_accept_label_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_file_current_folder_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_file_modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub save_file_response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub save_file_uris_listbox: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub save_files_title_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_files_accept_label_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_files_current_folder_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_files_modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub save_files_files_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_files_response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub save_files_uris_listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileChooserPage {
        const NAME: &'static str = "FileChooserPage";
        type Type = super::FileChooserPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "file_chooser.open_file",
                None,
                move |page, _action, _target| {
                    page.open_file();
                },
            );
            klass.install_action(
                "file_chooser.save_file",
                None,
                move |page, _action, _target| {
                    page.save_file();
                },
            );
            klass.install_action(
                "file_chooser.save_files",
                None,
                move |page, _action, _target| {
                    page.save_files();
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
}

glib::wrapper! {
    pub struct FileChooserPage(ObjectSubclass<imp::FileChooserPage>) @extends gtk::Widget, adw::Bin;
}

impl FileChooserPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a FileChooserPage")
    }

    fn open_file(&self) {
        let root = self.native().unwrap();
        spawn!(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let self_ = imp::FileChooserPage::from_instance(&page);
            let title = self_.open_title_entry.text();
            let accept_label = is_empty(self_.open_accept_label_entry.text());
            let directory = self_.open_directory_switch.is_active();
            let modal = self_.open_modal_switch.is_active();
            let multiple = self_.open_multiple_switch.is_active();

            if let Ok(files) = portal_open_file(&identifier, &title, accept_label.as_deref(), directory, modal, multiple).await {
                self_.open_response_group.show();

                while let Some(child) = self_.open_uris_listbox.next_sibling() {
                    self_.open_uris_listbox.remove(&child);
                }
                for uri in files.uris() {
                    self_.open_uris_listbox.append(&adw::ActionRow::builder().title(uri).build());
                }
            }
        }));
    }

    fn save_file(&self) {
        let root = self.native().unwrap();
        spawn!(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let self_ = imp::FileChooserPage::from_instance(&page);
            let title = self_.save_file_title_entry.text();
            let accept_label = is_empty(self_.save_file_accept_label_entry.text());
            let modal = self_.save_file_modal_switch.is_active();
            let current_name = is_empty(self_.save_file_current_name_entry.text());
            let current_folder = is_empty(self_.save_file_current_folder_entry.text());
            let current_file = is_empty(self_.save_file_current_file_entry.text());

            if let Ok(files) = portal_save_file(&identifier, &title, accept_label.as_deref(), modal, current_name.as_deref(), current_folder.as_deref(), current_file.as_deref()).await {
                self_.save_file_response_group.show();

                while let Some(child) = self_.save_file_uris_listbox.next_sibling() {
                    self_.save_file_uris_listbox.remove(&child);
                }
                for uri in files.uris() {
                    self_.save_file_uris_listbox.append(&adw::ActionRow::builder().title(uri).build());
                }
            }
        }));
    }

    fn save_files(&self) {
        let root = self.native().unwrap();
        spawn!(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let self_ = imp::FileChooserPage::from_instance(&page);
            let title = self_.save_files_title_entry.text();
            let accept_label = is_empty(self_.save_files_accept_label_entry.text());
            let current_folder = is_empty(self_.save_files_current_folder_entry.text());
            let modal = self_.save_files_modal_switch.is_active();
            let files = is_empty(self_.save_files_files_entry.text()).map(|files| split_comma(files));

            if let Ok(files) = portal_save_files(&identifier, &title, accept_label.as_deref(), modal, current_folder.as_deref(), files.as_deref()).await {
                self_.save_files_response_group.show();

                while let Some(child) = self_.save_files_uris_listbox.next_sibling() {
                    self_.save_files_uris_listbox.remove(&child);
                }
                for uri in files.uris() {
                    self_.save_files_uris_listbox.append(&adw::ActionRow::builder().title(uri).build());
                }
            }
        }));
    }
}

async fn portal_open_file(
    identifier: &WindowIdentifier,
    title: &str,
    accept_label: Option<&str>,
    directory: bool,
    modal: bool,
    multiple: bool,
) -> ashpd::Result<SelectedFiles> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = FileChooserProxy::new(&cnx).await?;
    let options = OpenFileOptions::default()
        .directory(directory)
        .modal(modal)
        .multiple(multiple);
    let options = if let Some(accept_label) = accept_label {
        options.accept_label(accept_label)
    } else {
        options
    };

    let selected_files = proxy.open_file(&identifier, title, options).await?;
    Ok(selected_files)
}

async fn portal_save_file(
    identifier: &WindowIdentifier,
    title: &str,
    accept_label: Option<&str>,
    modal: bool,
    current_name: Option<&str>,
    current_folder: Option<&str>,
    current_file: Option<&str>,
) -> ashpd::Result<SelectedFiles> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = FileChooserProxy::new(&cnx).await?;
    let options = SaveFileOptions::default().modal(modal);
    let options = if let Some(accept_label) = accept_label {
        options.accept_label(accept_label)
    } else {
        options
    };
    let options = if let Some(current_name) = current_name {
        options.current_name(current_name)
    } else {
        options
    };
    let options = if let Some(current_folder) = current_folder {
        options.current_folder(current_folder)
    } else {
        options
    };
    let options = if let Some(current_file) = current_file {
        options.current_file(current_file)
    } else {
        options
    };
    let selected_files = proxy.save_file(&identifier, title, options).await?;
    Ok(selected_files)
}

async fn portal_save_files<S: AsRef<str>>(
    identifier: &WindowIdentifier,
    title: &str,
    accept_label: Option<&str>,
    modal: bool,
    current_folder: Option<&str>,
    files: Option<&[S]>,
) -> ashpd::Result<SelectedFiles> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = FileChooserProxy::new(&cnx).await?;
    let options = SaveFilesOptions::default().modal(modal);
    let options = if let Some(accept_label) = accept_label {
        options.accept_label(accept_label)
    } else {
        options
    };
    let options = if let Some(current_folder) = current_folder {
        options.current_folder(current_folder)
    } else {
        options
    };
    let options = if let Some(files) = files {
        options.files(
            files
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
    } else {
        options
    };
    let selected_files = proxy.save_files(&identifier, title, options).await?;
    Ok(selected_files)
}
