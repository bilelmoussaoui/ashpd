use std::sync::Arc;

use adw::subclass::prelude::*;
use ashpd::flatpak::update_monitor::{UpdateMonitor, UpdateProgress, UpdateStatus};
use formatx::formatx;
use gettextrs::gettext;
use gtk::{
    glib::{self, clone},
    prelude::*,
};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/update_window.ui")]
    #[properties(wrapper_type = super::UpdateWindow)]
    pub struct UpdateWindow {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,

        #[property(get, set, nullable)]
        pub update_description: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        pub progress_description: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        pub error_description: RefCell<Option<String>>,

        pub monitor: RefCell<Option<Arc<UpdateMonitor>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UpdateWindow {
        const NAME: &'static str = "UpdateWindow";
        type Type = super::UpdateWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action_async("update.install", None, |win, _, _| async move {
                let root = win.native().unwrap();
                let window_identifier = ashpd::WindowIdentifier::from_native(&root).await;
                win.start_install(window_identifier);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for UpdateWindow {}
    impl WidgetImpl for UpdateWindow {}
    impl WindowImpl for UpdateWindow {}
    impl AdwWindowImpl for UpdateWindow {}
}

glib::wrapper! {
    pub struct UpdateWindow(ObjectSubclass<imp::UpdateWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl UpdateWindow {
    pub fn new(
        parent: &impl IsA<gtk::Window>,
        monitor: Option<Arc<UpdateMonitor>>,
        running_commit: Option<&str>,
        remote_commit: Option<&str>,
    ) -> Self {
        let window: Self = glib::Object::builder()
            .property("transient-for", parent)
            .build();

        let imp = window.imp();
        imp.monitor.replace(monitor);

        // Set the update description
        if let Some(running_commit) = running_commit
            && let Some(remote_commit) = remote_commit
        {
            let description = format!(
                "A new version is available.\n\nRunning: {}\nAvailable: {}",
                if running_commit.len() >= 8 {
                    &running_commit[..8]
                } else {
                    running_commit
                },
                if remote_commit.len() >= 8 {
                    &remote_commit[..8]
                } else {
                    remote_commit
                }
            );
            window.set_update_description(Some(description));
        }
        // Show the available page
        imp.stack.set_visible_child_name("available");

        window
    }

    fn start_install(&self, window_identifier: Option<ashpd::WindowIdentifier>) {
        let imp = self.imp();
        let Some(monitor) = imp.monitor.borrow().clone() else {
            return;
        };

        // Switch to installing page
        imp.stack.set_visible_child_name("installing");
        self.set_progress_description(Some(gettext("Starting update…")));
        imp.progress_bar.set_fraction(0.0);

        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<Result<UpdateProgress, String>>(10);

        // Spawn glib task to receive progress updates
        glib::spawn_future_local(clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                while let Some(result) = receiver.recv().await {
                    win.handle_progress(result);
                }
            }
        ));

        // Spawn tokio task to start update and listen for progress
        crate::portals::RUNTIME.spawn(async move {
            if let Err(err) = monitor
                .update(window_identifier.as_ref(), Default::default())
                .await
            {
                tracing::error!("Failed to start update: {err}");
                let _ = sender
                    .send(Err(format!("Failed to start update: {err}")))
                    .await;
                return;
            }

            let mut stream = match monitor.receive_progress().await {
                Ok(stream) => stream,
                Err(err) => {
                    tracing::error!("Failed to receive progress stream: {err}");
                    let _ = sender
                        .send(Err(format!("Failed to receive progress: {err}")))
                        .await;
                    return;
                }
            };

            use futures_util::StreamExt;
            while let Some(progress) = stream.next().await {
                if sender.send(Ok(progress)).await.is_err() {
                    break;
                }
            }
        });
    }

    fn handle_progress(&self, result: Result<UpdateProgress, String>) {
        let imp = self.imp();

        match result {
            Ok(progress) => {
                let status = progress.status().unwrap_or(UpdateStatus::Running);

                match status {
                    UpdateStatus::Running => {
                        let op = progress.op().unwrap_or(0);
                        let n_ops = progress.n_ops().unwrap_or(0);
                        let pct = progress.progress().unwrap_or(0);

                        self.set_progress_description(Some(
                            formatx!(gettext("Operation {} of {}"), op, n_ops).unwrap(),
                        ));

                        if n_ops > 0 {
                            imp.progress_bar.set_fraction(pct as f64 / 100.0);
                        } else {
                            imp.progress_bar.pulse();
                        }
                    }
                    UpdateStatus::Done => {
                        imp.stack.set_visible_child_name("done");
                    }
                    UpdateStatus::Failed => {
                        let error_msg = progress.error_message().unwrap_or("Unknown error");
                        self.set_error_description(Some(error_msg.to_string()));
                        imp.stack.set_visible_child_name("failed");
                    }
                    UpdateStatus::Empty => {
                        self.set_error_description(Some("No update to install.".to_string()));
                        imp.stack.set_visible_child_name("failed");
                    }
                }
            }
            Err(error_msg) => {
                self.set_error_description(Some(error_msg));
                imp.stack.set_visible_child_name("failed");
            }
        }
    }
}
