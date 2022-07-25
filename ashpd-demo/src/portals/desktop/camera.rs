use std::os::unix::prelude::RawFd;

use ashpd::{desktop::camera, zbus};
use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::widgets::{
    CameraPaintable, NotificationKind, PortalPage, PortalPageExt, PortalPageImpl,
};

mod imp {
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/camera.ui")]
    pub struct CameraPage {
        #[template_child]
        pub camera_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub picture_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub picture_stack_switcher: TemplateChild<gtk::StackSwitcher>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,

        pub paintables: RefCell<Vec<CameraPaintable>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraPage {
        const NAME: &'static str = "CameraPage";
        type Type = super::CameraPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("camera.start", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.start_stream().await;
                }));
            });
            klass.install_action("camera.stop", None, move |page, _, _| {
                page.stop_stream();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for CameraPage {
        fn constructed(&self, obj: &Self::Type) {
            obj.action_set_enabled("camera.stop", false);
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for CameraPage {
        fn map(&self, widget: &Self::Type) {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak widget as page => async move {
                let imp = page.imp();
                let is_available = camera_available().await.unwrap_or(false);
                if is_available {
                    imp.camera_available.set_text("Yes");
                    page.action_set_enabled("camera.start", true);
                } else {
                    imp.camera_available.set_text("No");

                    page.action_set_enabled("camera.start", false);
                    page.action_set_enabled("camera.stop", false);
                }
            }));
            self.parent_map(widget);
        }
    }
    impl BinImpl for CameraPage {}
    impl PortalPageImpl for CameraPage {}
}

glib::wrapper! {
    pub struct CameraPage(ObjectSubclass<imp::CameraPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl CameraPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPage")
    }

    async fn start_stream(&self) {
        let imp = self.imp();

        self.action_set_enabled("camera.stop", true);
        self.action_set_enabled("camera.start", false);
        match stream().await {
            Ok(stream_fd) => {
                let streams = camera::pipewire_streams(stream_fd).await.unwrap();
                let n_cameras = streams.len();
                for s in streams.iter() {
                    let picture = gtk::Picture::new();
                    let paintable = CameraPaintable::new();
                    let props = s.properties();
                    let nick = props
                        .get("node.nick")
                        .or(props.get("node.description"))
                        .or(props.get("node.name"))
                        .map(String::from)
                        .unwrap_or_default();
                    let name = props.get("name").map(String::from).unwrap_or_default();
                    tracing::debug!(
                        "Found camera. node_id: {}, nick: {nick}, name: {name}",
                        s.node_id()
                    );

                    picture.set_vexpand(true);
                    picture.set_paintable(Some(&paintable));
                    paintable.set_pipewire_node_id(stream_fd, Some(s.node_id()));

                    self.imp()
                        .picture_stack
                        .add_titled(&picture, Some(&name), &nick);
                    self.imp().paintables.borrow_mut().push(paintable.clone());
                }
                imp.revealer.set_reveal_child(n_cameras > 0);
                imp.picture_stack_switcher.set_visible(n_cameras > 1);
                if n_cameras > 0 {
                    imp.camera_available.set_text("Yes");
                } else {
                    imp.camera_available.set_text("No");
                }

                self.send_notification(
                    "Camera stream started successfully",
                    NotificationKind::Success,
                );
            }
            Err(err) => {
                tracing::error!("Failed to start a camera stream {:#?}", err);
                self.send_notification(
                    "Request to start a camera stream failed",
                    NotificationKind::Error,
                );
                self.stop_stream();
            }
        }
    }

    fn stop_stream(&self) {
        let imp = self.imp();
        self.action_set_enabled("camera.stop", false);
        self.action_set_enabled("camera.start", true);

        for paintable in imp.paintables.take() {
            paintable.close_pipeline();
        }
        while let Some(child) = imp.picture_stack.first_child() {
            imp.picture_stack.remove(&child);
        }

        imp.revealer.set_reveal_child(false);
    }
}

async fn stream() -> ashpd::Result<RawFd> {
    let proxy = camera::CameraProxy::new().await?;
    proxy.access_camera().await?;
    Ok(proxy.open_pipe_wire_remote().await?)
}

async fn camera_available() -> ashpd::Result<bool> {
    let proxy = camera::CameraProxy::new().await?;
    Ok(proxy.is_camera_present().await?)
}
