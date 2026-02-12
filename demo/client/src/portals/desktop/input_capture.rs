use std::sync::Arc;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    WindowIdentifier,
    desktop::{
        Session,
        input_capture::{Barrier, BarrierID, Capabilities, InputCapture},
    },
};
use async_channel;
use futures_util::{StreamExt, lock::Mutex};
use gtk::glib;

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/input_capture.ui")]
    pub struct InputCapturePage {
        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,
        pub session: Arc<Mutex<Option<Session<InputCapture>>>>,
        pub activation_id: Arc<Mutex<Option<u32>>>,
        pub barrier_positions: Arc<Mutex<HashMap<u32, BarrierPosition>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InputCapturePage {
        const NAME: &'static str = "InputCapturePage";
        type Type = super::InputCapturePage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("input_capture.start", None, |page, _, _| async move {
                page.start_session().await;
            });
            klass.install_action_async("input_capture.stop", None, |page, _, _| async move {
                page.stop_session().await;
            });
            klass.install_action_async("input_capture.enable", None, |page, _, _| async move {
                page.enable().await;
            });
            klass.install_action_async("input_capture.disable", None, |page, _, _| async move {
                page.disable().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for InputCapturePage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.action_set_enabled("input_capture.stop", false);
            obj.action_set_enabled("input_capture.enable", false);
            obj.action_set_enabled("input_capture.disable", false);
        }
    }
    impl WidgetImpl for InputCapturePage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { InputCapture::new().await }).await {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for InputCapturePage {}
    impl PortalPageImpl for InputCapturePage {}
}

glib::wrapper! {
    pub struct InputCapturePage(ObjectSubclass<imp::InputCapturePage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

pub struct BarrierPosition {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl BarrierPosition {
    pub fn edge_name(&self) -> String {
        if self.x1 == self.x2 {
            if self.x1 == 0 {
                "Left".to_string()
            } else {
                "Right".to_string()
            }
        } else if self.y1 == self.y2 {
            if self.y1 == 0 {
                "Top".to_string()
            } else {
                "Bottom".to_string()
            }
        } else {
            "Unknown".to_string()
        }
    }
}

impl From<(i32, i32, i32, i32)> for BarrierPosition {
    fn from(pos: (i32, i32, i32, i32)) -> Self {
        BarrierPosition {
            x1: pos.0,
            y1: pos.1,
            x2: pos.2,
            y2: pos.3,
        }
    }
}

impl InputCapturePage {
    async fn start_session(&self) {
        let imp = self.imp();

        self.action_set_enabled("input_capture.start", false);
        self.action_set_enabled("input_capture.stop", true);

        self.info("Starting input capture session");
        match self.create_session().await {
            Ok(session) => {
                self.success("Input capture session started successfully");

                if let Some(old_session) = imp.session.lock().await.replace(session) {
                    spawn_tokio(async move {
                        let _ = old_session.close().await;
                    })
                    .await;
                }

                imp.status_label.set_text("Started");
                self.action_set_enabled("input_capture.enable", true);
            }
            Err(err) => {
                tracing::error!("Failed to start input capture session: {err}");
                self.error(&format!("Failed to start input capture session: {err}"));
                self.stop_session().await;
            }
        }
    }

    async fn stop_session(&self) {
        self.action_set_enabled("input_capture.start", true);
        self.action_set_enabled("input_capture.stop", false);
        self.action_set_enabled("input_capture.enable", false);
        self.action_set_enabled("input_capture.disable", false);

        let imp = self.imp();
        if let Some(session) = imp.session.lock().await.take() {
            spawn_tokio(async move {
                let _ = session.close().await;
            })
            .await;
        }

        imp.status_label.set_text("Not active");
        imp.barrier_positions.lock().await.clear();

        self.info("Input capture session stopped");
    }

    async fn enable(&self) {
        let imp = self.imp();

        let proxy = match spawn_tokio(InputCapture::new()).await {
            Ok(p) => p,
            Err(err) => {
                tracing::error!("Failed to create InputCapture proxy: {err}");
                self.error(&format!("Failed to create InputCapture proxy: {err}"));
                return;
            }
        };

        let session_guard = imp.session.lock().await;
        if let Some(session) = session_guard.as_ref() {
            match proxy.enable(session).await {
                Ok(_) => {
                    self.success("Input capture enabled - move cursor to screen edge");
                    imp.status_label.set_text("Enabled");
                    self.action_set_enabled("input_capture.enable", false);
                    self.action_set_enabled("input_capture.disable", true);

                    // Listen for activation signals
                    let widget = self.clone();
                    glib::spawn_future_local(async move {
                        widget.listen_for_activation().await;
                    });
                }
                Err(err) => {
                    tracing::error!("Failed to enable input capture: {err}");
                    self.error(&format!("Failed to enable input capture: {err}"));
                }
            }
        }
    }

    async fn disable(&self) {
        let imp = self.imp();

        let proxy = match spawn_tokio(InputCapture::new()).await {
            Ok(p) => p,
            Err(err) => {
                tracing::error!("Failed to create InputCapture proxy: {err}");
                self.error(&format!("Failed to create InputCapture proxy: {err}"));
                return;
            }
        };

        let session_guard = imp.session.lock().await;
        if let Some(session) = session_guard.as_ref() {
            match proxy.disable(session).await {
                Ok(_) => {
                    self.success("Input capture disabled");
                    self.action_set_enabled("input_capture.enable", true);
                    self.action_set_enabled("input_capture.disable", false);
                    imp.status_label.set_text("Disabled");
                }
                Err(err) => {
                    tracing::error!("Failed to disable input capture: {err}");
                    self.error(&format!("Failed to disable input capture: {err}"));
                }
            }
        }
    }

    async fn listen_for_activation(&self) {
        let imp = self.imp();

        let session_guard = imp.session.lock().await;
        if session_guard.is_none() {
            return;
        }
        drop(session_guard);

        let widget = self.clone();
        let (sender, receiver_glib) =
            async_channel::unbounded::<ashpd::desktop::input_capture::Activated>();

        // Bridge from tokio to glib main loop
        glib::spawn_future_local(async move {
            while let Ok(activated) = receiver_glib.recv().await {
                let barrier_base_text = match activated.barrier_id() {
                    Some(ashpd::desktop::input_capture::ActivatedBarrier::Barrier(id)) => {
                        // Look up the barrier position and calculate the name
                        let barrier_positions = widget.imp().barrier_positions.lock().await;
                        if let Some(position) = barrier_positions.get(&id.get()) {
                            position.edge_name()
                        } else {
                            format!("Barrier {}", id)
                        }
                    }
                    Some(ashpd::desktop::input_capture::ActivatedBarrier::UnknownBarrier) => {
                        "Unknown barrier".to_string()
                    }
                    None => "Barrier".to_string(),
                };

                widget.info(&format!("{} activated!", barrier_base_text));

                // Store activation_id for release
                if let Some(id) = activated.activation_id() {
                    *widget.imp().activation_id.lock().await = Some(id);
                }

                // Countdown from 5 to 0
                let widget_clone = widget.clone();
                let barrier_text = barrier_base_text.clone();
                glib::spawn_future_local(async move {
                    for remaining in (1..=5).rev() {
                        widget_clone.imp().status_label.set_text(&format!(
                            "{} activated, releasing in {}s",
                            barrier_text, remaining,
                        ));
                        glib::timeout_future_seconds(1).await;
                    }

                    // Release and re-enable the Enable button
                    widget_clone.release().await;
                    widget_clone.imp().status_label.set_text("Disabled");
                    widget_clone.action_set_enabled("input_capture.enable", true);
                    widget_clone.action_set_enabled("input_capture.disable", false);
                });
            }
        });

        crate::portals::RUNTIME.spawn(async move {
            let proxy = match InputCapture::new().await {
                Ok(p) => p,
                Err(err) => {
                    tracing::error!("Failed to create InputCapture proxy: {err}");
                    return;
                }
            };

            let mut activated_stream = match proxy.receive_activated().await {
                Ok(stream) => stream,
                Err(err) => {
                    tracing::error!("Failed to receive activation stream: {err}");
                    return;
                }
            };

            while let Some(activated) = activated_stream.next().await {
                if sender.send(activated).await.is_err() {
                    break;
                }
            }
        });
    }

    async fn release(&self) {
        let imp = self.imp();

        let proxy = match spawn_tokio(InputCapture::new()).await {
            Ok(p) => p,
            Err(err) => {
                tracing::error!("Failed to create InputCapture proxy: {err}");
                return;
            }
        };

        let session_guard = imp.session.lock().await;
        let activation_id = imp.activation_id.lock().await.take();

        if let Some(session) = session_guard.as_ref() {
            match proxy.release(session, activation_id, None).await {
                Ok(_) => {
                    self.info("Input capture released");
                }
                Err(err) => {
                    tracing::error!("Failed to release input capture: {err}");
                    self.error(&format!("Failed to release input capture: {err}"));
                }
            }
        }
    }

    async fn create_session(&self) -> ashpd::Result<Session<InputCapture>> {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let (session, failed_barriers, eis_result, barrier_positions) = spawn_tokio(async move {
            let proxy = InputCapture::new().await?;

            // Create session with all capabilities. We don't let the user select the capabilities
            // because... meh?
            let capabilities =
                Capabilities::Keyboard | Capabilities::Pointer | Capabilities::Touchscreen;
            let (session, _caps) = proxy
                .create_session(identifier.as_ref(), capabilities)
                .await?;

            let zones_response = proxy.zones(&session).await?;
            let zones = zones_response.response()?;
            let (barriers, barrier_positions) = calculate_outside_barriers(zones.regions());

            let failed_barriers = match proxy
                .set_pointer_barriers(&session, &barriers, zones.zone_set())
                .await
            {
                Ok(response) => match response.response() {
                    Ok(result) => result.failed_barriers().to_vec(),
                    Err(err) => {
                        tracing::error!("Failed to parse barrier response: {err}");
                        vec![]
                    }
                },
                Err(err) => {
                    tracing::error!("Failed to set barriers: {err}");
                    vec![]
                }
            };

            // Connect to EIS, we don't actually care about EI events though
            let eis_result = proxy.connect_to_eis(&session).await;

            ashpd::Result::Ok((session, failed_barriers, eis_result, barrier_positions))
        })
        .await?;

        // Store barrier positions for later use
        *self.imp().barrier_positions.lock().await = barrier_positions;

        if !failed_barriers.is_empty() {
            self.info(&format!("Some barriers failed: {:?}", failed_barriers));
        } else {
            self.info("All barriers set successfully");
        }

        match eis_result {
            Ok(_fd) => {
                self.info("Connected to EIS");
                // We don't use EI in this demo, so we don't care if the fd gets closed
            }
            Err(err) => {
                tracing::error!("Failed to connect to EIS: {err}");
                self.error(&format!("Failed to connect to EIS: {err}"));
            }
        }

        Ok(session)
    }
}

/// Calculate barriers on outside edges.
/// Returns (barriers, barrier_positions) where barrier_positions maps barrier ID to position
fn calculate_outside_barriers(
    regions: &[ashpd::desktop::input_capture::Region],
) -> (
    Vec<Barrier>,
    std::collections::HashMap<u32, BarrierPosition>,
) {
    use std::collections::{HashMap, HashSet};

    if regions.is_empty() {
        return (vec![], HashMap::new());
    }

    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    enum Edge {
        Top(i32, i32, i32, i32),    // x1, y1, x2, y2
        Bottom(i32, i32, i32, i32), // x1, y1, x2, y2
        Left(i32, i32, i32, i32),   // x1, y1, x2, y2
        Right(i32, i32, i32, i32),  // x1, y1, x2, y2
    }

    let mut all_edges = HashSet::new();
    let mut shared_edges = HashSet::new();

    // Collect all edges
    for region in regions {
        let x = region.x_offset();
        let y = region.y_offset();
        let width = region.width() as i32;
        let height = region.height() as i32;

        let edges = [
            Edge::Top(x, y, x + width, y),
            Edge::Bottom(x, y + height, x + width, y + height),
            Edge::Left(x, y, x, y + height),
            Edge::Right(x + width, y, x + width, y + height),
        ];

        for edge in edges {
            if !all_edges.insert(edge) {
                shared_edges.insert(edge);
            }
        }
    }

    let mut barriers = Vec::new();
    let mut barrier_positions = HashMap::new();
    let mut barrier_id = 1u32;

    for edge in all_edges.iter().filter(|e| !shared_edges.contains(e)) {
        let position = match *edge {
            Edge::Top(x1, y1, x2, y2) => (x1, y1, x2 - 1, y2),
            Edge::Bottom(x1, y1, x2, y2) => (x1, y1, x2 - 1, y2),
            Edge::Left(x1, y1, x2, y2) => (x1, y1, x2, y2 - 1),
            Edge::Right(x1, y1, x2, y2) => (x1, y1, x2, y2 - 1),
        };

        if let Some(id) = BarrierID::new(barrier_id) {
            barriers.push(Barrier::new(id, position));
            let pos = BarrierPosition::from(position);
            barrier_positions.insert(barrier_id, pos);
            barrier_id = barrier_id.saturating_add(1);
        } else {
            tracing::warn!(
                "Failed to create barrier ID {}, stopping barrier creation",
                barrier_id
            );
            break;
        }
    }

    (barriers, barrier_positions)
}
