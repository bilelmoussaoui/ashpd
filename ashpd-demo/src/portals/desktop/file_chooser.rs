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
            let accept_label = self_.open_accept_label_entry.text();
            let directory = self_.open_directory_switch.is_active();
            let modal = self_.open_modal_switch.is_active();
            let multiple = self_.open_multiple_switch.is_active();

            let files = portal_open_file(identifier, &title, &accept_label, directory, modal, multiple).await.unwrap();
            self_.open_response_group.show();

            while let Some(child) = self_.open_uris_listbox.next_sibling() {
                self_.open_uris_listbox.remove(&child);
            }
            for uri in files.uris() {
                self_.open_uris_listbox.append(&adw::ActionRow::builder().title(uri).build());
            }
        }));
    }

    fn save_file(&self) {
        let root = self.native().unwrap();
        spawn!(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let self_ = imp::FileChooserPage::from_instance(&page);
            let title = self_.save_file_title_entry.text();
            let accept_label = self_.save_file_accept_label_entry.text();
            let modal = self_.save_file_modal_switch.is_active();
            let current_name = self_.save_file_current_name_entry.text();
            let current_folder = self_.save_file_current_folder_entry.text();
            let current_file = self_.save_file_current_file_entry.text();

            let files = portal_save_file(identifier, &title, &accept_label, modal, &current_name, &current_folder, &current_file).await.unwrap();
            self_.save_file_response_group.show();

            while let Some(child) = self_.save_file_uris_listbox.next_sibling() {
                self_.save_file_uris_listbox.remove(&child);
            }
            for uri in files.uris() {
                self_.save_file_uris_listbox.append(&adw::ActionRow::builder().title(uri).build());
            }
        }));
    }

    fn save_files(&self) {
        let root = self.native().unwrap();
        spawn!(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let self_ = imp::FileChooserPage::from_instance(&page);
            let title = self_.save_files_title_entry.text();
            let accept_label = self_.save_files_accept_label_entry.text();
            let current_folder = self_.save_files_current_folder_entry.text();
            let modal = self_.save_files_modal_switch.is_active();
            let files = self_.save_files_files_entry.text();
            let files = files.split(',').collect::<Vec<&str>>();

            let files = portal_save_files(identifier, &title, &accept_label, modal, &current_folder, files.as_slice()).await.unwrap();
            self_.save_files_response_group.show();

            while let Some(child) = self_.save_files_uris_listbox.next_sibling() {
                self_.save_files_uris_listbox.remove(&child);
            }
            for uri in files.uris() {
                self_.save_files_uris_listbox.append(&adw::ActionRow::builder().title(uri).build());
            }
        }));
    }
}

async fn portal_open_file(
    identifier: WindowIdentifier,
    title: &str,
    accept_label: &str,
    directory: bool,
    modal: bool,
    multiple: bool,
) -> Result<SelectedFiles, ashpd::Error> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = FileChooserProxy::new(&cnx).await?;
    let selected_files = proxy
        .open_file(
            identifier,
            title,
            OpenFileOptions::default()
                .accept_label(accept_label)
                .directory(directory)
                .modal(modal)
                .multiple(multiple),
        )
        .await?;
    Ok(selected_files)
}

async fn portal_save_file(
    identifier: WindowIdentifier,
    title: &str,
    accept_label: &str,
    modal: bool,
    current_name: &str,
    current_folder: &str,
    current_file: &str,
) -> Result<SelectedFiles, ashpd::Error> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = FileChooserProxy::new(&cnx).await?;
    let selected_files = proxy
        .save_file(
            identifier,
            title,
            SaveFileOptions::default()
                .accept_label(accept_label)
                .modal(modal)
                .current_name(current_name)
                .current_folder(current_folder)
                .current_file(current_file),
        )
        .await?;
    Ok(selected_files)
}

async fn portal_save_files(
    identifier: WindowIdentifier,
    title: &str,
    accept_label: &str,
    modal: bool,
    current_folder: &str,
    files: &[&str],
) -> Result<SelectedFiles, ashpd::Error> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = FileChooserProxy::new(&cnx).await?;
    let selected_files = proxy
        .save_files(
            identifier,
            title,
            SaveFilesOptions::default()
                .accept_label(accept_label)
                .modal(modal)
                .current_folder(current_folder)
                .files(files),
        )
        .await?;
    Ok(selected_files)
}
