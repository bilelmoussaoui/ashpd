use ashpd::{desktop::camera, Response};
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
        #[template_child]
        pub start_session_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub close_session_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
    }

    impl Default for CameraPage {
        fn default() -> Self {
            Self {
                camera_available: TemplateChild::default(),
                picture: TemplateChild::default(),
                paintable: CameraPaintable::new(),
                start_session_btn: TemplateChild::default(),
                close_session_btn: TemplateChild::default(),
                revealer: TemplateChild::default(),
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
            klass.install_action("camera.start", None, move |page, _action, _target| {
                page.start_stream();
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
        fn constructed(&self, _obj: &Self::Type) {
            let ctx = glib::MainContext::default();
            let start_session_btn = self.start_session_btn.get();
            let camera_available = self.camera_available.get();
            ctx.spawn_local(clone!(@weak start_session_btn, @weak camera_available => async move {
                let is_present = camera::is_present().await.unwrap_or(false);
                if is_present {
                    camera_available.set_text("Yes");
                } else {
                    camera_available.set_text("No");
                }
                start_session_btn.set_sensitive(is_present);
            }));

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
        let ctx = glib::MainContext::default();
        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::CameraPage::from_instance(&page);

            if let Ok(Response::Ok(stream_fd)) = camera::stream().await {
                self_.paintable.set_pipewire_fd(stream_fd);
                self_.start_session_btn.set_sensitive(false);
                self_.close_session_btn.set_sensitive(true);
                self_.revealer.set_reveal_child(true);
            }
        }));
    }

    pub fn stop_stream(&self) {
        let self_ = imp::CameraPage::from_instance(self);

        self_.paintable.close_pipeline();
        self_.close_session_btn.set_sensitive(false);
        self_.start_session_btn.set_sensitive(true);
        self_.revealer.set_reveal_child(false);
    }
}
