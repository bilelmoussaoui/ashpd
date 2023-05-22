use std::os::unix::prelude::RawFd;

use adw::subclass::prelude::*;
use ashpd::desktop::camera;
use gtk::{
    glib::{self, clone},
    prelude::*,
};

use crate::widgets::{CameraPaintable, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
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
            klass.bind_template();

            klass.install_action_async("camera.start", None, |page, _, _| async move {
                page.start_stream().await;
            });
            klass.install_action("camera.stop", None, |page, _, _| {
                page.stop_stream();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for CameraPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().action_set_enabled("camera.stop", false);
        }
    }
    impl WidgetImpl for CameraPage {
        fn map(&self) {
            let ctx = glib::MainContext::default();
            let widget = self.obj();
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
            self.parent_map();
        }
    }
    impl BinImpl for CameraPage {}
    impl PortalPageImpl for CameraPage {}
}

glib::wrapper! {
    pub struct CameraPage(ObjectSubclass<imp::CameraPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl CameraPage {
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
                    let paintable = CameraPaintable::default();
                    let props = s.properties();
                    let nick = props
                        .get("node.nick")
                        .or_else(|| props.get("node.description"))
                        .or_else(|| props.get("node.name"))
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

                self.success("Camera stream started successfully");
            }
            Err(err) => {
                tracing::error!("Failed to start a camera stream: {err}");
                self.error("Request to start a camera stream failed");
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
    let proxy = camera::Camera::new().await?;
    proxy.request_access().await?;
    proxy.open_pipe_wire_remote().await
}

async fn camera_available() -> ashpd::Result<bool> {
    let proxy = camera::Camera::new().await?;
    proxy.is_present().await
}
