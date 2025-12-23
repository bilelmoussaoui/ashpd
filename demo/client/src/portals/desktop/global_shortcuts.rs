use std::{collections::HashSet, sync::Arc};

use adw::subclass::prelude::*;
use ashpd::{
    WindowIdentifier,
    desktop::{
        ResponseError, Session,
        global_shortcuts::{
            Activated, Deactivated, GlobalShortcuts, NewShortcut, Shortcut, ShortcutsChanged,
        },
    },
};
use futures_util::lock::Mutex;
use gtk::{
    glib::{self, clone},
    prelude::*,
};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

#[derive(Debug, Clone)]
pub struct RegisteredShortcut {
    id: String,
    activation: String,
}

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/global_shortcuts.ui")]
    pub struct GlobalShortcutsPage {
        #[template_child]
        pub shortcuts: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub activations_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub activations_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub rebind_count_label: TemplateChild<gtk::Label>,
        pub rebind_count: Arc<Mutex<u32>>,
        pub session: Arc<Mutex<Option<Session<GlobalShortcuts>>>>,
        pub task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
        pub triggers: Arc<Mutex<Vec<RegisteredShortcut>>>,
        pub activations: Arc<Mutex<HashSet<String>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GlobalShortcutsPage {
        const NAME: &'static str = "GlobalShortcutsPage";
        type Type = super::GlobalShortcutsPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async(
                "global_shortcuts.start_session",
                None,
                |page, _, _| async move {
                    if let Err(err) = page.start_session().await {
                        tracing::error!("Failed to request {}", err);
                    }
                },
            );
            klass.install_action_async("global_shortcuts.stop", None, |page, _, _| async move {
                page.stop().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for GlobalShortcutsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj()
                .action_set_enabled("global_shortcuts.stop", false);
        }
    }
    impl WidgetImpl for GlobalShortcutsPage {}
    impl BinImpl for GlobalShortcutsPage {}
    impl PortalPageImpl for GlobalShortcutsPage {}
}

glib::wrapper! {
    pub struct GlobalShortcutsPage(ObjectSubclass<imp::GlobalShortcutsPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl GlobalShortcutsPage {
    async fn start_session(&self) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;
        let shortcuts = imp.shortcuts.text();
        let shortcuts: Option<Vec<_>> = shortcuts
            .as_str()
            .split(',')
            .map(|desc| {
                let mut split = desc.splitn(3, ':');
                let name = split.next()?;
                let desc = split.next()?;
                let trigger = split.next();
                Some(NewShortcut::new(name, desc).preferred_trigger(trigger))
            })
            .collect();

        match shortcuts {
            Some(shortcuts) => {
                let result = spawn_tokio(async move {
                    let global_shortcuts = GlobalShortcuts::new().await?;
                    let session = global_shortcuts.create_session().await?;
                    let request = global_shortcuts
                        .bind_shortcuts(&session, &shortcuts[..], identifier.as_ref())
                        .await?;
                    let shortcuts_data = request
                        .response()
                        .map(|resp| {
                            resp.shortcuts()
                                .iter()
                                .map(|s: &Shortcut| RegisteredShortcut {
                                    id: s.id().to_owned(),
                                    activation: s.trigger_description().to_owned(),
                                })
                                .collect::<Vec<_>>()
                        })
                        .map_err(|e| match e {
                            ashpd::Error::Response(ResponseError::Cancelled) => "Cancelled",
                            ashpd::Error::Response(ResponseError::Other) => "Other response error",
                            _ => "Unknown error",
                        });
                    ashpd::Result::Ok((global_shortcuts, session, shortcuts_data))
                })
                .await?;
                let (global_shortcuts, session, shortcuts_data) = result;

                if let Err(e) = &shortcuts_data {
                    self.error(e);
                };
                imp.activations_group.set_visible(shortcuts_data.is_ok());
                self.action_set_enabled("global_shortcuts.stop", shortcuts_data.is_ok());
                self.action_set_enabled("global_shortcuts.start_session", shortcuts_data.is_err());
                self.imp().shortcuts.set_editable(shortcuts_data.is_err());
                match shortcuts_data {
                    Ok(triggers) => {
                        *imp.triggers.lock().await = triggers;
                        self.display_activations().await;
                        self.set_rebind_count(Some(0));

                        if let Some(old_session) = imp.session.lock().await.replace(session) {
                            spawn_tokio(async move {
                                let _ = old_session.close().await;
                            })
                            .await;
                        }

                        let (activated_sender, activated_rx) = async_channel::unbounded();
                        let (deactivated_sender, deactivated_rx) = async_channel::unbounded();
                        let (changed_sender, changed_rx) = async_channel::unbounded();

                        glib::spawn_future_local(clone!(
                            #[weak(rename_to = page)]
                            self,
                            async move {
                                while let Ok(activation) = activated_rx.recv().await {
                                    page.on_activated(activation).await;
                                }
                            }
                        ));

                        glib::spawn_future_local(clone!(
                            #[weak(rename_to = page)]
                            self,
                            async move {
                                while let Ok(deactivation) = deactivated_rx.recv().await {
                                    page.on_deactivated(deactivation).await;
                                }
                            }
                        ));

                        glib::spawn_future_local(clone!(
                            #[weak(rename_to = page)]
                            self,
                            async move {
                                while let Ok(change) = changed_rx.recv().await {
                                    page.on_changed(change).await;
                                }
                            }
                        ));

                        let task_handle = crate::portals::RUNTIME.spawn(async move {
                            Self::track_incoming_events_task(
                                global_shortcuts,
                                activated_sender,
                                deactivated_sender,
                                changed_sender,
                            )
                            .await;
                        });
                        if let Some(old_handle) = imp.task_handle.lock().await.replace(task_handle)
                        {
                            old_handle.abort();
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failure {:?}", e);
                    }
                }
            }
            _ => {
                self.error("Shortcut list invalid");
            }
        };

        Ok(())
    }

    fn set_rebind_count(&self, count: Option<u32>) {
        let label = &self.imp().rebind_count_label;
        match count {
            None => label.set_text(""),
            Some(count) => label.set_text(&format!("{}", count)),
        }
    }

    async fn track_incoming_events_task(
        global_shortcuts: GlobalShortcuts,
        activated_sender: async_channel::Sender<Activated>,
        deactivated_sender: async_channel::Sender<Deactivated>,
        changed_sender: async_channel::Sender<ShortcutsChanged>,
    ) {
        use futures_util::StreamExt;

        let gs = Arc::new(global_shortcuts);
        let gs1 = gs.clone();
        let gs2 = gs.clone();
        let gs3 = gs;

        let (activated_tx, mut activated_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Result<Activated, ashpd::Error>>();
        let (deactivated_tx, mut deactivated_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Result<Deactivated, ashpd::Error>>();
        let (changed_tx, mut changed_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Result<ShortcutsChanged, ashpd::Error>>();

        // Spawn task for activated events
        crate::portals::RUNTIME.spawn(async move {
            if let Ok(mut stream) = gs1.receive_activated().await {
                while let Some(activation) = stream.next().await {
                    if activated_tx.send(Ok(activation)).is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn task for deactivated events
        crate::portals::RUNTIME.spawn(async move {
            if let Ok(mut stream) = gs2.receive_deactivated().await {
                while let Some(deactivation) = stream.next().await {
                    if deactivated_tx.send(Ok(deactivation)).is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn task for shortcuts changed events
        crate::portals::RUNTIME.spawn(async move {
            if let Ok(mut stream) = gs3.receive_shortcuts_changed().await {
                while let Some(changed) = stream.next().await {
                    if changed_tx.send(Ok(changed)).is_err() {
                        break;
                    }
                }
            }
        });

        loop {
            tokio::select! {
                Some(result) = activated_receiver.recv() => {
                    match result {
                        Ok(activation) => {
                            if activated_sender.send(activation).await.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            tracing::error!("Activated stream error: {err}");
                            break;
                        }
                    }
                }
                Some(result) = deactivated_receiver.recv() => {
                    match result {
                        Ok(deactivation) => {
                            if deactivated_sender.send(deactivation).await.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            tracing::error!("Deactivated stream error: {err}");
                            break;
                        }
                    }
                }
                Some(result) = changed_receiver.recv() => {
                    match result {
                        Ok(change) => {
                            if changed_sender.send(change).await.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            tracing::error!("Changed stream error: {err}");
                            break;
                        }
                    }
                }
                else => break,
            }
        }
    }

    async fn stop(&self) {
        let imp = self.imp();
        self.action_set_enabled("global_shortcuts.stop", false);
        self.action_set_enabled("global_shortcuts.start_session", true);
        self.imp().shortcuts.set_editable(true);

        if let Some(handle) = self.imp().task_handle.lock().await.take() {
            handle.abort();
        }

        if let Some(session) = imp.session.lock().await.take() {
            spawn_tokio(async move {
                let _ = session.close().await;
            })
            .await;
        }
        imp.activations_group.set_visible(false);
        self.set_rebind_count(None);
        imp.activations.lock().await.clear();
        imp.triggers.lock().await.clear();
    }

    async fn display_activations(&self) {
        let activations = self.imp().activations.lock().await.clone();
        let triggers = self.imp().triggers.lock().await.clone();
        let text: Vec<String> = triggers
            .into_iter()
            .map(|RegisteredShortcut { id, activation }| {
                let escape = |s: &str| glib::markup_escape_text(s).to_string();
                let id = escape(&id);
                let activation = escape(&activation);
                if activations.contains(&id) {
                    format!("<b>{}: {}</b>", id, activation)
                } else {
                    format!("{}: {}", id, activation)
                }
            })
            .collect();
        self.imp().activations_label.set_markup(&text.join("\n"))
    }

    async fn on_activated(&self, activation: Activated) {
        {
            let mut activations = self.imp().activations.lock().await;
            activations.insert(activation.shortcut_id().into());
        }
        self.display_activations().await
    }

    async fn on_deactivated(&self, deactivation: Deactivated) {
        {
            let mut activations = self.imp().activations.lock().await;
            if !activations.remove(deactivation.shortcut_id()) {
                tracing::warn!(
                    "Received deactivation without previous activation: {:?}",
                    deactivation
                );
            }
        }
        self.display_activations().await
    }

    async fn on_changed(&self, change: ShortcutsChanged) {
        *self.imp().triggers.lock().await = change
            .shortcuts()
            .iter()
            .map(|s| RegisteredShortcut {
                id: s.id().to_owned(),
                activation: s.trigger_description().to_owned(),
            })
            .collect();
        *self.imp().rebind_count.lock().await += 1;
        self.set_rebind_count(Some(*self.imp().rebind_count.lock().await));
        self.display_activations().await
    }
}
