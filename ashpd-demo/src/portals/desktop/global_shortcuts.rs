use std::{collections::HashSet, sync::Arc};

use adw::subclass::prelude::*;
use ashpd::{
    desktop::{
        global_shortcuts::{Activated, Deactivated, ShortcutsChanged, GlobalShortcuts, NewShortcut, Shortcut},
        ResponseError,
        Session,
    },
    WindowIdentifier,
};
use gtk::{glib, prelude::*};
use futures_util::{
    future::{AbortHandle, Abortable},
    lock::Mutex,
    stream::{select_all, Stream, StreamExt},
};
use crate::widgets::{PortalPage, PortalPageExt, PortalPageImpl};

#[derive(Debug)]
enum Event {
    Activated(Activated),
    Deactivated(Deactivated),
    ShortcutsChanged(ShortcutsChanged),
}

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
        pub session: Arc<Mutex<Option<Session<'static>>>>,
        pub abort_handle: Arc<Mutex<Option<AbortHandle>>>,
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

            klass.install_action_async("global_shortcuts.start_session", None, |page, _, _| async move {
                if let Err(err) = page.start_session().await {
                    tracing::error!("Failed to request {}", err);
                }
            });
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
            self.obj().action_set_enabled("global_shortcuts.stop", false);
        }
    }
    impl WidgetImpl for GlobalShortcutsPage {}
    impl BinImpl for GlobalShortcutsPage {}
    impl PortalPageImpl for GlobalShortcutsPage {}
}

glib::wrapper! {
    pub struct GlobalShortcutsPage(ObjectSubclass<imp::GlobalShortcutsPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl GlobalShortcutsPage {
    async fn start_session(&self) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;
        let shortcuts = imp.shortcuts.text();
        let shortcuts: Option<Vec<_>> = shortcuts.as_str().split(',')
            .map(|desc| {
                let mut split = desc.splitn(3, ':');
                let name = split.next()?;
                let desc = split.next()?;
                let trigger = split.next();
                Some(NewShortcut::new(name, desc).preferred_trigger(trigger))
            }).collect();

        match shortcuts {
            Some(shortcuts) => {
                let global_shortcuts = GlobalShortcuts::new().await?;
                let session = global_shortcuts.create_session().await?;
                let request = global_shortcuts.bind_shortcuts(&session, &shortcuts[..], &identifier).await?;
                let response = request.response();
                if let Err(e) = &response {
                    self.error(&match e {
                        ashpd::Error::Response(ResponseError::Cancelled) => "Cancelled".into(),
                        ashpd::Error::Response(ResponseError::Other) => "Other response error".into(),
                        other => format!("{}", other),
                    })
                };
                imp.activations_group.set_visible(response.is_ok());
                self.action_set_enabled("global_shortcuts.stop", response.is_ok());
                self.action_set_enabled("global_shortcuts.start_session", !response.is_ok());
                self.imp().shortcuts.set_editable(!response.is_ok());
                match response {
                    Ok(resp) => {
                        let triggers: Vec<_>
                            = resp.shortcuts().iter()
                            .map(|s: &Shortcut| RegisteredShortcut {
                                id: s.id().to_owned(),
                                activation: s.trigger_description().to_owned(),
                            })
                            .collect();
                        *imp.triggers.lock().await = triggers;
                        self.display_activations().await;
                        self.set_rebind_count(Some(0));
                        imp.session.lock().await.replace(session);
                        loop {
                            if imp.session.lock().await.is_none() {
                                break;
                            }

                            let (abort_handle, abort_registration) = AbortHandle::new_pair();
                            let future = Abortable::new(
                                self.track_incoming_events(&global_shortcuts),
                                abort_registration,
                            );
                            imp.abort_handle.lock().await.replace(abort_handle);
                            let _ = future.await;
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failure {:?}", e);
                    }
                }
            },
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

    async fn track_incoming_events(&self, global_shortcuts: &GlobalShortcuts<'_>) {
        let Ok(activated_stream) = global_shortcuts.receive_activated().await
        else {
            return;
        };
        let Ok(deactivated_stream) = global_shortcuts.receive_deactivated().await
        else {
            return;
        };
        let Ok(changed_stream) = global_shortcuts.receive_shortcuts_changed().await
        else {
            return;
        };

        let bact: Box<dyn Stream<Item=Event> + Unpin> = Box::new(activated_stream.map(Event::Activated));
        let bdeact: Box<dyn Stream<Item=Event> + Unpin> = Box::new(deactivated_stream.map(Event::Deactivated));
        let bchg: Box<dyn Stream<Item=Event> + Unpin> = Box::new(changed_stream.map(Event::ShortcutsChanged));

        let mut events = select_all([
            bact, bdeact, bchg,
        ]);

        while let Some(event) = events.next().await {
            match event {
                Event::Activated(activation) => {
                    self.on_activated(activation).await;
                },
                Event::Deactivated(deactivation) => {
                    self.on_deactivated(deactivation).await;
                },
                Event::ShortcutsChanged(change) => {
                    self.on_changed(change).await;
                },
            }
        }
    }

    async fn stop(&self) {
        let imp = self.imp();
        self.action_set_enabled("global_shortcuts.stop", false);
        self.action_set_enabled("global_shortcuts.start_session", true);
        self.imp().shortcuts.set_editable(true);

        if let Some(abort_handle) = self.imp().abort_handle.lock().await.take() {
            abort_handle.abort();
        }

        if let Some(session) = imp.session.lock().await.take() {
            let _ = session.close().await;
        }
        imp.activations_group.set_visible(false);
        self.set_rebind_count(None);
        imp.activations.lock().await.clear();
        imp.triggers.lock().await.clear();
    }

    async fn display_activations(&self) {
        let activations = self.imp().activations.lock().await.clone();
        let triggers = self.imp().triggers.lock().await.clone();
        let text: Vec<String> = triggers.into_iter()
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
                tracing::warn!("Received deactivation without previous activation: {:?}", deactivation);
            }
        }
        self.display_activations().await
    }

    async fn on_changed(&self, change: ShortcutsChanged) {
        *self.imp().triggers.lock().await
            = change.shortcuts().iter()
                .map(|s| RegisteredShortcut{
                    id: s.id().to_owned(),
                    activation: s.trigger_description().to_owned(),
                })
                .collect();
        *self.imp().rebind_count.lock().await += 1;
        self.set_rebind_count(Some(*self.imp().rebind_count.lock().await));
        self.display_activations().await
    }
}
