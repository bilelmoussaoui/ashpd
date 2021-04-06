use std::os::unix::io;
use std::os::unix::io::AsRawFd;
use std::{collections::HashMap, sync::Arc};

use ashpd::zbus;
use ashpd::{
    desktop::camera::{AsyncCameraProxy, CameraAccessOptions, CameraProxy},
    BasicResponse, Response,
};
use futures::{lock::Mutex, FutureExt};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::widgets::CameraPaintable;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/camera.ui")]
    pub struct CameraPage {
        #[template_child]
        pub camera_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        pub paintable: CameraPaintable,
    }

    impl Default for CameraPage {
        fn default() -> Self {
            Self {
                camera_available: TemplateChild::default(),
                picture: TemplateChild::default(),
                paintable: CameraPaintable::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraPage {
        const NAME: &'static str = "CameraPage";
        type Type = super::CameraPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "camera.start-stream",
                None,
                move |page, _action, _target| {
                    page.start_stream();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for CameraPage {
        fn constructed(&self, _obj: &Self::Type) {
            let connection = zbus::Connection::new_session().unwrap();
            let camera_proxy = CameraProxy::new(&connection).unwrap();
            let camera_available = camera_proxy.is_camera_present().unwrap();

            self.camera_available
                .set_text(&camera_available.to_string());
            self.picture.set_paintable(Some(&self.paintable));
        }
    }
    impl WidgetImpl for CameraPage {}
    impl BinImpl for CameraPage {}
}

glib::wrapper! {
    pub struct CameraPage(ObjectSubclass<imp::CameraPage>) @extends gtk::Widget, adw::Bin;
}

impl CameraPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPage")
    }

    pub fn start_stream(&self) {
        let self_ = imp::CameraPage::from_instance(self);

        let ctx = glib::MainContext::default();
        let paintable = &self_.paintable;
        ctx.spawn_local(clone!(@weak paintable => async move {
            if let Ok(Response::Ok(stream_fd)) = start_stream().await {
                println!("{:#?}", stream_fd);
                paintable.set_pipewire_fd(stream_fd);
            }
        }));
    }
}

pub async fn start_stream() -> zbus::Result<Response<io::RawFd>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncCameraProxy::new(&connection)?;
    let request = proxy.access_camera(CameraAccessOptions::default()).await?;

    let (request_sender, request_receiver) = futures::channel::oneshot::channel();
    let request_sender = Arc::new(Mutex::new(Some(request_sender)));
    let request_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = request_sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(request_id).await?;

    if let Response::Err(err) = request_receiver.await.unwrap() {
        return Ok(Response::Err(err));
    }
    let remote_fd = proxy.open_pipe_wire_remote(HashMap::new()).await?;
    Ok(Response::Ok(remote_fd.as_raw_fd()))
}
