use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    WindowIdentifier,
    desktop::file_chooser::{
        Choice, FileFilter, OpenFileRequest, SaveFileRequest, SaveFilesRequest,
    },
};
use gtk::glib;

use self::choice_widget::ChoiceWidget;
use crate::{
    portals::{is_empty, spawn_tokio, split_comma},
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod choice_widget {
    use super::*;

    mod imp {
        use std::sync::OnceLock;

        use adw::subclass::prelude::BinImpl;
        use glib::subclass::Signal;

        use super::*;

        #[derive(Debug, Default)]
        pub struct ChoiceWidget {
            pub(super) id_row: adw::EntryRow,
            pub(super) label_row: adw::EntryRow,
            pub(super) type_combo: adw::ComboRow,
            pub(super) boolean_switch: adw::SwitchRow,
            pub(super) options_text: gtk::TextView,
            pub(super) initial_entry: adw::EntryRow,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for ChoiceWidget {
            const NAME: &'static str = "ChoiceWidget";
            type Type = super::ChoiceWidget;
            type ParentType = adw::Bin;
        }

        impl ObjectImpl for ChoiceWidget {
            fn signals() -> &'static [Signal] {
                static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
                SIGNALS.get_or_init(|| vec![Signal::builder("removed").action().build()])
            }

            fn constructed(&self) {
                self.parent_constructed();
                self.obj().create_widgets();
            }
        }
        impl WidgetImpl for ChoiceWidget {}
        impl BinImpl for ChoiceWidget {}
    }

    glib::wrapper! {
        pub struct ChoiceWidget(ObjectSubclass<imp::ChoiceWidget>)
            @extends gtk::Widget, adw::Bin,
            @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
    }

    impl ChoiceWidget {
        pub fn connect_removed<F>(&self, callback: F) -> glib::SignalHandlerId
        where
            F: Fn(&Self) + 'static,
        {
            self.connect_closure(
                "removed",
                false,
                glib::closure_local!(move |obj: &Self| {
                    callback(obj);
                }),
            )
        }

        fn get_options(&self) -> Vec<(String, String)> {
            let imp = self.imp();
            let buffer = imp.options_text.buffer();
            let (start, end) = buffer.bounds();
            let text = buffer.text(&start, &end, false);

            text.lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() {
                        return None;
                    }
                    line.split_once('=')
                        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                })
                .collect()
        }

        pub fn choice(&self) -> Choice {
            let imp = self.imp();
            let id = imp.id_row.text();
            let label = imp.label_row.text();

            match imp.type_combo.selected() {
                0 => {
                    // Boolean
                    let initial_state = imp.boolean_switch.is_active();
                    Choice::boolean(&id, &label, initial_state)
                }
                1 => {
                    // Dropdown
                    let initial = imp.initial_entry.text();
                    let mut choice = Choice::new(&id, &label, &initial);

                    for (key, value) in self.get_options() {
                        choice = choice.insert(&key, &value);
                    }
                    choice
                }
                _ => Choice::boolean(&id, &label, false),
            }
        }

        fn create_widgets(&self) {
            let imp = self.imp();
            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();

            let list_box = gtk::ListBox::new();
            list_box.add_css_class("boxed-list");

            imp.id_row.set_title("Choice ID");
            list_box.append(&imp.id_row);

            imp.label_row.set_title("Label");
            list_box.append(&imp.label_row);

            imp.type_combo.set_title("Type");
            imp.type_combo
                .set_model(Some(&gtk::StringList::new(&["Boolean", "Dropdown"])));
            imp.type_combo.connect_selected_notify(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |combo| {
                    let is_boolean = combo.selected() == 0;
                    obj.imp().boolean_switch.set_visible(is_boolean);
                    obj.imp().initial_entry.set_visible(!is_boolean);
                }
            ));
            list_box.append(&imp.type_combo);

            imp.boolean_switch.set_title("Initial State");
            imp.boolean_switch.set_visible(true); // Boolean is default
            list_box.append(&imp.boolean_switch);

            imp.initial_entry.set_title("Initial Selection");
            imp.initial_entry.set_text("utf8");
            imp.initial_entry.set_visible(false); // Hidden by default
            list_box.append(&imp.initial_entry);

            container.append(&list_box);

            // Options section for dropdown
            let options_expander = adw::ExpanderRow::builder()
                .title("Options")
                .subtitle("One option per line: key=value")
                .visible(false) // Hidden by default
                .build();

            imp.options_text.set_margin_start(12);
            imp.options_text.set_margin_end(12);
            imp.options_text.set_margin_top(6);
            imp.options_text.set_margin_bottom(6);
            imp.options_text.set_wrap_mode(gtk::WrapMode::Word);

            // Pre-fill with example options
            let buffer = imp.options_text.buffer();
            buffer.set_text("utf8=Unicode (UTF-8)\nlatin1=Western (ISO-8859-1)\nascii=ASCII");

            let scrolled = gtk::ScrolledWindow::builder()
                .child(&imp.options_text)
                .hscrollbar_policy(gtk::PolicyType::Never)
                .vscrollbar_policy(gtk::PolicyType::Automatic)
                .min_content_height(100)
                .build();

            options_expander.add_row(&scrolled);
            list_box.append(&options_expander);

            // Connect type combo to show/hide options expander
            imp.type_combo.connect_selected_notify(glib::clone!(
                #[weak]
                options_expander,
                move |combo| {
                    let is_dropdown = combo.selected() == 1;
                    options_expander.set_visible(is_dropdown);
                }
            ));

            let remove_button = gtk::Button::builder()
                .halign(gtk::Align::End)
                .valign(gtk::Align::Center)
                .margin_top(6)
                .label("Remove")
                .margin_bottom(12)
                .build();
            remove_button.add_css_class("destructive-action");
            remove_button.connect_clicked(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_btn| {
                    obj.emit_by_name::<()>("removed", &[]);
                }
            ));
            container.append(&remove_button);

            self.set_child(Some(&container));
        }
    }

    impl Default for ChoiceWidget {
        fn default() -> Self {
            glib::Object::new()
        }
    }
}

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
        pub open_modal_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub open_multiple_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub open_directory_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub open_filter_combo: TemplateChild<adw::ComboRow>,
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
        pub save_file_modal_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub save_file_filter_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub save_file_response_group: TemplateChild<adw::PreferencesGroup>,

        #[template_child]
        pub save_files_title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_accept_label_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_current_folder_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_modal_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub save_files_files_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub save_files_response_group: TemplateChild<adw::PreferencesGroup>,

        #[template_child]
        pub open_choices_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub save_file_choices_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub save_files_choices_box: TemplateChild<gtk::Box>,
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

            klass.install_action("file_chooser.add_open_choice", None, |page, _, _| {
                page.add_choice(&page.imp().open_choices_box);
            });
            klass.install_action("file_chooser.add_save_file_choice", None, |page, _, _| {
                page.add_choice(&page.imp().save_file_choices_box);
            });
            klass.install_action("file_chooser.add_save_files_choice", None, |page, _, _| {
                page.add_choice(&page.imp().save_files_choices_box);
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
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl FileChooserPage {
    fn add_choice(&self, container: &gtk::Box) {
        let choice = ChoiceWidget::default();
        choice.connect_removed(glib::clone!(
            #[weak]
            container,
            move |widget| {
                container.remove(widget);
            }
        ));
        container.append(&choice);
    }

    fn get_choices(&self, container: &gtk::Box) -> Vec<Choice> {
        let mut choices = Vec::new();
        let mut child = match container.first_child() {
            Some(c) => c,
            None => return choices,
        };

        loop {
            if let Some(choice_widget) = child.downcast_ref::<ChoiceWidget>() {
                choices.push(choice_widget.choice());
            }

            if let Some(next_child) = child.next_sibling() {
                child = next_child;
            } else {
                break;
            }
        }
        choices
    }

    fn filters(&self, pos: u32) -> Vec<FileFilter> {
        let mut filters = Vec::new();

        if pos > 0 {
            filters.push(
                FileFilter::new("Text files")
                    .mimetype("text/*")
                    .glob("*.txt"),
            );
        };
        if pos > 1 {
            filters.push(FileFilter::new("Images").mimetype("image/*"));
        };
        if pos > 2 {
            filters.push(FileFilter::new("Videos").mimetype("video/*"));
        };
        filters
    }

    async fn open_file(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let title = imp.open_title_entry.text();
        let directory = imp.open_directory_switch.is_active();
        let modal = imp.open_modal_switch.is_active();
        let multiple = imp.open_multiple_switch.is_active();
        let accept_label = is_empty(imp.open_accept_label_entry.text());

        let filters = self.filters(imp.open_filter_combo.selected());
        let current_filter = filters.first().cloned();
        let choices = self.get_choices(&imp.open_choices_box);
        let response = spawn_tokio(async move {
            let request = OpenFileRequest::default()
                .directory(directory)
                .identifier(identifier)
                .modal(modal)
                .title(&*title)
                .multiple(multiple)
                .filters(filters)
                .current_filter(current_filter)
                .accept_label(accept_label.as_deref())
                .choices(choices);
            request.send().await.and_then(|r| r.response())
        })
        .await;
        match response {
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
        let filters = self.filters(imp.save_file_filter_combo.selected());
        let current_filter = filters.first().cloned();
        let choices = self.get_choices(&imp.save_file_choices_box);
        let response = spawn_tokio(async move {
            let request = SaveFileRequest::default()
                .identifier(identifier)
                .modal(modal)
                .title(&*title)
                .filters(filters)
                .current_filter(current_filter)
                .accept_label(accept_label.as_deref())
                .current_name(current_name.as_deref())
                .current_folder::<String>(current_folder)?
                .current_file::<String>(current_file)?
                .choices(choices);
            request.send().await.and_then(|r| r.response())
        })
        .await;

        match response {
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
        let choices = self.get_choices(&imp.save_files_choices_box);
        let response = spawn_tokio(async move {
            let request = SaveFilesRequest::default()
                .identifier(identifier)
                .modal(modal)
                .title(&*title)
                .accept_label(accept_label.as_deref())
                .current_folder::<String>(current_folder)?
                .files::<Vec<_>>(files)?
                .choices(choices);

            request.send().await.and_then(|r| r.response())
        })
        .await;

        match response {
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
