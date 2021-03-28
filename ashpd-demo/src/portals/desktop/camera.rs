use ashpd::zbus;
use ashpd::{
    desktop::camera::{CameraAccessOptions, CameraProxy},
    desktop::device::{AccessDeviceOptions, Device, DeviceProxy},
    BasicResponse,
};
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use adw::subclass::prelude::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/camera.ui")]
    pub struct CameraPage {
        #[template_child]
        pub camera_available: TemplateChild<gtk::Label>,
        pub connection: zbus::Connection,
    }

    impl Default for CameraPage {
        fn default() -> Self {
            Self {
                camera_available: TemplateChild::default(),
                connection: zbus::Connection::new_session().unwrap(),
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
                    page.start_stream().unwrap();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for CameraPage {
        fn constructed(&self, obj: &Self::Type) {
            let camera_proxy = CameraProxy::new(&self.connection).unwrap();
            let camera_available = camera_proxy.is_camera_present().unwrap();

            self.camera_available
                .set_text(&camera_available.to_string());
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

    pub fn start_stream(&self) -> zbus::fdo::Result<()> {
        let self_ = imp::CameraPage::from_instance(self);

        let proxy = CameraProxy::new(&self_.connection)?;
        let request = proxy.access_camera(CameraAccessOptions::default())?;

        // let fd = proxy.open_pipe_wire_remote(std::collections::HashMap::new())?;
        request.connect_response(|response: Response<BasicResponse>| {
            println!("{:#?}", response);
            if let Response::Ok(info) = response {
                println!("{:#?}", info);
            }
            Ok(())
        })?;
        Ok(())
    }
}
