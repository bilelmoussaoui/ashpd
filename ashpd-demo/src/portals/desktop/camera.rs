use std::os::unix::prelude::RawFd;

use ashpd::{desktop::camera, zbus};
use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::widgets::{
    CameraPaintable, NotificationKind, PortalPage, PortalPageExt, PortalPageImpl,
};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/camera.ui")]
    pub struct CameraPage {
        #[template_child]
        pub camera_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        pub paintable: CameraPaintable,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
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
            self.picture.set_paintable(Some(&self.paintable));
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
                let node_id = camera::pipewire_node_id(stream_fd).await.unwrap();
                imp.paintable.set_pipewire_node_id(stream_fd, node_id);
                imp.revealer.set_reveal_child(true);
                imp.camera_available.set_text("Yes");

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

        imp.paintable.close_pipeline();
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
