use std::{convert::TryFrom, sync::Arc};

use adw::prelude::*;
use ashpd::zbus;
use ashpd::{
    desktop::location::{
        Accuracy, AsyncLocationProxy, CreateSessionOptions, Location, SessionStartOptions,
    },
    BasicResponse, HandleToken, Response, WindowIdentifier,
};
use futures::{lock::Mutex, FutureExt};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/location.ui")]
    pub struct LocationPage {
        #[template_child]
        pub accuracy_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub distance_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub time_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub accuracy_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub altitude_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub speed_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub heading_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub latitude_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub longitude_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub timestamp_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocationPage {
        const NAME: &'static str = "LocationPage";
        type Type = super::LocationPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("location.locate", None, move |page, _action, _target| {
                page.locate();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for LocationPage {
        fn constructed(&self, _obj: &Self::Type) {
            let model = gtk::StringList::new(&[
                "None",
                "Country",
                "City",
                "Neighborhood",
                "Street",
                "Exact",
            ]);
            self.accuracy_combo.set_model(Some(&model));
            self.accuracy_combo.set_selected(Accuracy::Exact as u32);
        }
    }
    impl WidgetImpl for LocationPage {}
    impl BinImpl for LocationPage {}
}

glib::wrapper! {
    pub struct LocationPage(ObjectSubclass<imp::LocationPage>) @extends gtk::Widget, adw::Bin;
}

impl LocationPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a LocationPage")
    }

    pub fn locate(&self) {
        let ctx = glib::MainContext::default();
        let self_ = imp::LocationPage::from_instance(self);
        let distance_threshold = self_.distance_spin.get_value() as u32;
        let time_threshold = self_.time_spin.get_value() as u32;
        let accuracy = unsafe { std::mem::transmute(self_.accuracy_combo.get_selected()) };
        let root = self.get_root().unwrap();

        ctx.spawn_local(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_window(&root).await;
            if let Ok(Response::Ok(location)) = locate(identifier, distance_threshold, time_threshold, accuracy).await {
                let self_ = imp::LocationPage::from_instance(&page);

                self_.response_group.show();
                self_.accuracy_label.set_label(&location.accuracy().to_string());
                self_.altitude_label.set_label(&location.altitude().to_string());
                self_.speed_label.set_label(&location.speed().to_string());
                self_.heading_label.set_label(&location.heading().to_string());
                self_.description_label.set_label(&location.description());
                self_.latitude_label.set_label(&location.latitude().to_string());
                self_.longitude_label.set_label(&location.longitude().to_string());
                self_.timestamp_label.set_label(&location.timestamp().to_string());
            }
        }));
    }
}

pub async fn locate(
    window_identifier: WindowIdentifier,
    distance_threshold: u32,
    time_threshold: u32,
    accuracy: Accuracy,
) -> zbus::Result<Response<Location>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncLocationProxy::new(&connection);
    let session = proxy
        .create_session(
            CreateSessionOptions::default()
                .session_handle_token(HandleToken::try_from("sometokenstuff").unwrap())
                .distance_threshold(distance_threshold)
                .time_threshold(time_threshold)
                .accuracy(accuracy),
        )
        .await?;

    let request = proxy
        .start(&session, window_identifier, SessionStartOptions::default())
        .await?;

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
    if let Response::Err(err) = request_receiver.await.unwrap() {
        return Ok(Response::Err(err));
    }

    let (location_sender, location_receiver) = futures::channel::oneshot::channel();
    let location_sender = Arc::new(Mutex::new(Some(location_sender)));
    let signal_id = proxy
        .connect_location_updated(move |location| {
            let s = location_sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(location);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while proxy.next_signal().await?.is_some() {}
    proxy.disconnect_signal(signal_id).await?;
    request.disconnect_signal(request_id).await?;
    session.close().await?;

    let location = location_receiver.await.unwrap();
    Ok(Response::Ok(location))
}
