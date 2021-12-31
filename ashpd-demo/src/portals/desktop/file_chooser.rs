use crate::portals::{is_empty, split_comma};
use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};
use adw::prelude::*;
use ashpd::{
    desktop::file_chooser::{
        FileChooserProxy, OpenFileOptions, SaveFileOptions, SaveFilesOptions, SelectedFiles,
    },
    zbus, WindowIdentifier,
};
use glib::clone;
use gtk::glib;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
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
        let accept_label = is_empty(imp.open_accept_label_entry.text());
        let directory = imp.open_directory_switch.is_active();
        let modal = imp.open_modal_switch.is_active();
        let multiple = imp.open_multiple_switch.is_active();

        match portal_open_file(
            &identifier,
            &title,
            accept_label.as_deref(),
            directory,
            modal,
            multiple,
        )
        .await
        {
            Ok(files) => {
                imp.open_response_group.show();

                for uri in files.uris() {
                    imp.open_response_group
                        .add(&adw::ActionRow::builder().title(uri).build());
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
        let accept_label = is_empty(imp.save_file_accept_label_entry.text());
        let modal = imp.save_file_modal_switch.is_active();
        let current_name = is_empty(imp.save_file_current_name_entry.text());
        let current_folder = is_empty(imp.save_file_current_folder_entry.text());
        let current_file = is_empty(imp.save_file_current_file_entry.text());

        match portal_save_file(
            &identifier,
            &title,
            accept_label.as_deref(),
            modal,
            current_name.as_deref(),
            current_folder.as_deref(),
            current_file.as_deref(),
        )
        .await
        {
            Ok(files) => {
                imp.save_file_response_group.show();

                for uri in files.uris() {
                    imp.save_file_response_group
                        .add(&adw::ActionRow::builder().title(uri).build());
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
        let accept_label = is_empty(imp.save_files_accept_label_entry.text());
        let current_folder = is_empty(imp.save_files_current_folder_entry.text());
        let modal = imp.save_files_modal_switch.is_active();
        let files = is_empty(imp.save_files_files_entry.text()).map(split_comma);

        match portal_save_files(
            &identifier,
            &title,
            accept_label.as_deref(),
            modal,
            current_folder.as_deref(),
            files.as_deref(),
        )
        .await
        {
            Ok(files) => {
                imp.save_files_response_group.show();

                for uri in files.uris() {
                    imp.save_files_response_group
                        .add(&adw::ActionRow::builder().title(uri).build());
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

async fn portal_open_file(
    identifier: &WindowIdentifier,
    title: &str,
    accept_label: Option<&str>,
    directory: bool,
    modal: bool,
    multiple: bool,
) -> ashpd::Result<SelectedFiles> {
    let cnx = zbus::Connection::session().await?;
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

    let selected_files = proxy.open_file(identifier, title, options).await?;
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
    let cnx = zbus::Connection::session().await?;
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
    let selected_files = proxy.save_file(identifier, title, options).await?;
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
    let cnx = zbus::Connection::session().await?;
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
    let selected_files = proxy.save_files(identifier, title, options).await?;
    Ok(selected_files)
}
