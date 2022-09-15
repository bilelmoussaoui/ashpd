use std::path::Path;

use ashpd::{
    backend::{
        file_chooser::{
            FileChooserImpl, OpenFileOptions, OpenFileResults, SaveFileOptions, SaveFileResults,
            SaveFilesOptions, SaveFilesResults,
        },
        request::RequestImpl,
    },
    desktop::{
        file_chooser::{Choice, FileFilter},
        Response,
    },
    AppID, ExternalWindow, WindowIdentifierType,
};
use async_trait::async_trait;
use byteorder::LE;
use futures_channel::oneshot;
use gettextrs::gettext;
use gtk::{gio, gio::prelude::*, glib, prelude::*};
use zbus::zvariant::{from_slice, EncodingContext};

pub enum FileChooserAction {
    OpenFile(
        String,
        WindowIdentifierType,
        OpenFileOptions,
        oneshot::Sender<Response<OpenFileResults>>,
    ),
    SaveFile(
        String,
        WindowIdentifierType,
        SaveFileOptions,
        oneshot::Sender<Response<SaveFileResults>>,
    ),
    SaveFiles(
        String,
        WindowIdentifierType,
        SaveFilesOptions,
        oneshot::Sender<Response<SaveFilesResults>>,
    ),
}

#[derive(Default)]

pub struct FileChooser;
#[async_trait]
impl RequestImpl for FileChooser {
    async fn close(&self) {
        log::debug!("IN Close()");
    }
}

#[async_trait]
impl FileChooserImpl for FileChooser {
    async fn open_file(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: &str,
        options: OpenFileOptions,
    ) -> Response<OpenFileResults> {
        log::debug!("IN OpenFile({app_id}, {window_identifier}, {title}, {options:?}");

        let (sender, receiver) = oneshot::channel::<Response<OpenFileResults>>();

        let response = open_file_dialog(&title, window_identifier, options).await;
        sender.send(response).unwrap();

        let response = receiver.await.unwrap();
        log::debug!("OUT OpenFile({response:?})");

        response
    }

    async fn save_file(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: &str,
        options: SaveFileOptions,
    ) -> Response<SaveFileResults> {
        log::debug!("IN SaveFile({app_id}, {window_identifier}, {title}, {options:?}");

        let (sender, receiver) = oneshot::channel::<Response<SaveFileResults>>();

        let response = save_file_dialog(&title, window_identifier, options).await;
        sender.send(response).unwrap();

        let response = receiver.await.unwrap();
        log::debug!("OUT SaveFile({response:?})");

        response
    }

    async fn save_files(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: &str,
        options: SaveFilesOptions,
    ) -> Response<SaveFilesResults> {
        log::debug!("IN SaveFiles({app_id}, {window_identifier}, {title}, {options:?}");

        let (sender, receiver) = oneshot::channel::<Response<SaveFilesResults>>();

        let response = save_files_dialog(&title, window_identifier, options).await;
        sender.send(response).unwrap();

        let response = receiver.await.unwrap();
        log::debug!("OUT SaveFiles({response:?})");

        response
    }
}

async fn open_file_dialog(
    title: &str,
    window_identifier: WindowIdentifierType,
    options: OpenFileOptions,
) -> Response<OpenFileResults> {
    let multiple = options.multiple == Some(true);
    let is_directory = options.directory == Some(true);

    let mut choices = options.choices.clone().unwrap_or_default();

    let accept_label = if multiple {
        options.accept_label.unwrap_or_else(|| gettext("_Select"))
    } else {
        options.accept_label.unwrap_or_else(|| gettext("_Open"))
    };

    let action = if is_directory {
        gtk::FileChooserAction::SelectFolder
    } else {
        gtk::FileChooserAction::Open
    };

    let external_window = ExternalWindow::new(window_identifier);
    let fake_window = ExternalWindow::fake(external_window.as_ref());

    let file_chooser = gtk::FileChooserDialog::new(
        Some(title),
        Some(&fake_window),
        action,
        &[
            (&gettext("_Cancel"), gtk::ResponseType::Cancel),
            (&accept_label, gtk::ResponseType::Ok),
        ],
    );

    if let Some(ref external) = external_window {
        gtk::Widget::realize(file_chooser.upcast_ref());
        external.set_parent_of(&file_chooser.surface());
    }

    file_chooser.set_default_response(gtk::ResponseType::Ok);

    if let Some(filters) = options.filters {
        for filter in filters {
            let gfilter = gtk::FileFilter::new();

            let label = filter.label();
            if !label.is_empty() {
                gfilter.set_name(Some(filter.label()));
            }

            for mime in filter.mimetype_filters() {
                gfilter.add_mime_type(mime);
            }

            for pattern in filter.pattern_filters() {
                gfilter.add_pattern(pattern);
            }

            file_chooser.add_filter(&gfilter);

            if Some(filter) == options.current_filter {
                file_chooser.set_filter(&gfilter);
            }
        }
    }

    let modal = options.modal.unwrap_or(true);
    file_chooser.set_modal(modal);

    let choice_label = if is_directory {
        gettext("Open directories read-only")
    } else {
        gettext("Open files read-only")
    };
    choices.push(Choice::boolean("read-only", &choice_label, false));
    for choice in choices.iter() {
        let id = choice.id();
        let label = choice.label();
        let pairs = choice.pairs();
        let initial_selection = choice.initial_selection();
        file_chooser.add_choice(id, label, &pairs);
        file_chooser.set_choice(id, initial_selection);
    }

    let result = file_chooser.run_future().await;
    file_chooser.close();

    if result == gtk::ResponseType::Ok {
        let files = file_chooser.files();
        let uris: Vec<url::Url> = files
            .into_iter()
            .filter_map(Result::ok)
            .map(|file| file.downcast::<gio::File>().unwrap().uri())
            .filter_map(|x| url::Url::parse(&x).ok())
            .collect();

        let uris = if uris.is_empty() { None } else { Some(uris) };

        let current_filter = file_chooser.filter().and_then(|gfilter| {
            let variant = gfilter.to_gvariant();
            let ctxt = EncodingContext::<LE>::new_gvariant(0);
            let decoded: zbus::zvariant::Result<FileFilter> = from_slice(variant.data(), ctxt);

            decoded.ok()
        });

        // We get the values for the choices.
        let choices = choices
            .iter()
            .filter_map(|choice| {
                file_chooser.choice(choice.id()).map(|initial_selection| {
                    Choice::new(choice.id(), choice.label(), &initial_selection)
                })
            })
            .collect::<Vec<Choice>>();

        let writable = file_chooser.choice("read-only").map(|val| &val == "false");
        log::debug!("writable: {writable:?}");

        let res = OpenFileResults {
            uris,
            writable,
            current_filter,
            choices: Some(choices),
        };

        Response::ok(res)
    } else if result == gtk::ResponseType::Cancel {
        Response::cancelled()
    } else {
        Response::other()
    }
}

async fn save_file_dialog(
    title: &str,
    window_identifier: WindowIdentifierType,
    options: SaveFileOptions,
) -> Response<SaveFileResults> {
    let choices = options.choices.clone().unwrap_or_default();

    let accept_label = options.accept_label.unwrap_or_else(|| gettext("_Save"));

    let action = gtk::FileChooserAction::Save;

    let external_window = ExternalWindow::new(window_identifier);
    let fake_window = ExternalWindow::fake(external_window.as_ref());

    let file_chooser = gtk::FileChooserDialog::new(
        Some(title),
        Some(&fake_window),
        action,
        &[
            (&gettext("_Cancel"), gtk::ResponseType::Cancel),
            (&accept_label, gtk::ResponseType::Ok),
        ],
    );

    file_chooser.set_select_multiple(false);

    if let Some(ref external) = external_window {
        gtk::Widget::realize(file_chooser.upcast_ref());
        external.set_parent_of(&file_chooser.surface());
    }

    file_chooser.set_default_response(gtk::ResponseType::Ok);

    if let Some(filters) = options.filters {
        for filter in filters {
            let gfilter = gtk::FileFilter::new();

            let label = filter.label();
            if !label.is_empty() {
                gfilter.set_name(Some(filter.label()));
            }

            for mime in filter.mimetype_filters() {
                gfilter.add_mime_type(mime);
            }

            for pattern in filter.pattern_filters() {
                gfilter.add_pattern(pattern);
            }

            file_chooser.add_filter(&gfilter);

            if Some(filter) == options.current_filter {
                file_chooser.set_filter(&gfilter);
            }
        }
    }

    let modal = options.modal.unwrap_or(true);
    file_chooser.set_modal(modal);

    for choice in choices.iter() {
        let id = choice.id();
        let label = choice.label();
        let pairs = choice.pairs();
        let initial_selection = choice.initial_selection();
        file_chooser.add_choice(id, label, &pairs);
        file_chooser.set_choice(id, initial_selection);
    }

    if let Some(path) = options.current_file {
        let file = gio::File::for_path(path);
        let _ = file_chooser.set_file(&file);
    } else {
        if let Some(current_name) = options.current_name {
            file_chooser.set_current_name(&current_name);
        }

        if let Some(current_folder) = options.current_folder {
            let file = gio::File::for_path(current_folder);
            let _ = file_chooser.set_current_folder(Some(&file));
        }
    }

    let result = file_chooser.run_future().await;
    file_chooser.close();

    if result == gtk::ResponseType::Ok {
        let files = file_chooser.files();
        let uris: Vec<url::Url> = files
            .into_iter()
            .filter_map(Result::ok)
            .map(|file| file.downcast::<gio::File>().unwrap().uri())
            .filter_map(|x| url::Url::parse(&x).ok())
            .collect();

        let uris = if uris.is_empty() { None } else { Some(uris) };

        let current_filter = file_chooser.filter().and_then(|gfilter| {
            let variant = gfilter.to_gvariant();
            let ctxt = EncodingContext::<LE>::new_gvariant(0);
            let decoded: zbus::zvariant::Result<FileFilter> = from_slice(variant.data(), ctxt);

            decoded.ok()
        });

        // We get the values for the choices.
        let choices = choices
            .iter()
            .filter_map(|choice| {
                file_chooser.choice(choice.id()).map(|initial_selection| {
                    Choice::new(choice.id(), choice.label(), &initial_selection)
                })
            })
            .collect::<Vec<Choice>>();

        let writable = file_chooser.choice("read-only").map(|val| &val == "false");
        log::debug!("writable: {writable:?}");

        let res = SaveFileResults {
            uris,
            current_filter,
            choices: Some(choices),
        };

        Response::ok(res)
    } else if result == gtk::ResponseType::Cancel {
        Response::cancelled()
    } else {
        Response::other()
    }
}

async fn save_files_dialog(
    title: &str,
    window_identifier: WindowIdentifierType,
    options: SaveFilesOptions,
) -> Response<SaveFilesResults> {
    let mut choices = options.choices.clone().unwrap_or_default();

    let accept_label = options.accept_label.unwrap_or_else(|| gettext("_Save"));

    let action = gtk::FileChooserAction::SelectFolder;

    let external_window = ExternalWindow::new(window_identifier);
    let fake_window = ExternalWindow::fake(external_window.as_ref());

    let file_chooser = gtk::FileChooserDialog::new(
        Some(title),
        Some(&fake_window),
        action,
        &[
            (&gettext("_Cancel"), gtk::ResponseType::Cancel),
            (&accept_label, gtk::ResponseType::Ok),
        ],
    );

    file_chooser.set_select_multiple(false);

    file_chooser.set_default_response(gtk::ResponseType::Ok);

    if let Some(current_folder) = options.current_folder {
        let file = gio::File::for_path(current_folder);
        let _ = file_chooser.set_current_folder(Some(&file));
    }

    let modal = options.modal.unwrap_or(true);
    file_chooser.set_modal(modal);

    let choice_label = gettext("Open directories read-only");

    choices.push(Choice::boolean("read-only", &choice_label, false));
    for choice in choices.iter() {
        let id = choice.id();
        let label = choice.label();
        let pairs = choice.pairs();
        let initial_selection = choice.initial_selection();
        file_chooser.add_choice(id, label, &pairs);
        file_chooser.set_choice(id, initial_selection);
    }

    if let Some(ref external) = external_window {
        gtk::Widget::realize(file_chooser.upcast_ref());
        external.set_parent_of(&file_chooser.surface());
    }

    let result = file_chooser.run_future().await;
    file_chooser.close();

    if result == gtk::ResponseType::Ok {
        let mut uris = Vec::<url::Url>::new();
        if let Some(files) = options.files {
            let Some(base_dir) = file_chooser.file() else {
                return Response::other();
            };
            for file_name in files {
                let mut file = base_dir.child(file_name);
                let mut unique_id = 0;
                while file.query_exists(gio::Cancellable::NONE) {
                    // FIXME We don't support paths like a/b.txt.
                    unique_id += 1;
                    let Some(old_name) = file
                        .basename() else {
                            return Response::other();
                        };

                    let identifier = format!(" ({unique_id})");
                    // FIXME This is wrong, it splits a.tar.gz into [a.tar, gz]
                    // instead of [a, tar.gz].
                    let new_path = if let Some(file_name) = old_name.file_stem() {
                        let mut new_name = file_name.to_owned();
                        new_name.push(&identifier);
                        log::debug!("new name: {new_name:?}");
                        let parent = old_name.parent().unwrap_or_else(|| Path::new(""));
                        log::debug!("parent: {parent:?}");
                        if let Some(ext) = old_name.extension() {
                            parent.join(new_name).with_extension(ext)
                        } else {
                            parent.join(new_name)
                        }
                    } else {
                        let mut cloned = old_name.clone();
                        cloned.push(&identifier);
                        cloned
                    };

                    file = base_dir.child(new_path);
                }

                let Ok(uri) = url::Url::parse(&file.uri()) else {
                    return Response::other();
                };

                uris.push(uri)
            }
        }
        let uris = if uris.is_empty() { None } else { Some(uris) };

        // We get the values for the choices.
        let choices = choices
            .iter()
            .filter_map(|choice| {
                file_chooser.choice(choice.id()).map(|initial_selection| {
                    Choice::new(choice.id(), choice.label(), &initial_selection)
                })
            })
            .collect::<Vec<Choice>>();

        let writable = file_chooser.choice("read-only").map(|val| &val == "false");
        log::debug!("writable: {writable:?}");

        let res = SaveFilesResults {
            uris,
            choices: Some(choices),
        };

        Response::ok(res)
    } else if result == gtk::ResponseType::Cancel {
        Response::cancelled()
    } else {
        Response::other()
    }
}

mod convert {}
