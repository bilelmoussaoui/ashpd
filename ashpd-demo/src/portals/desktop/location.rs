use adw::prelude::*;
use ashpd::{
    desktop::location::{Accuracy, Location, LocationProxy},
    zbus, WindowIdentifier,
};
use futures::TryFutureExt;
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
    impl ObjectImpl for LocationPage {}
    impl WidgetImpl for LocationPage {}
    impl BinImpl for LocationPage {}
}

glib::wrapper! {
    pub struct LocationPage(ObjectSubclass<imp::LocationPage>) @extends gtk::Widget, adw::Bin;
}

impl LocationPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a LocationPage")
    }

    pub fn locate(&self) {
        let ctx = glib::MainContext::default();
        let self_ = imp::LocationPage::from_instance(self);
        let distance_threshold = self_.distance_spin.value() as u32;
        let time_threshold = self_.time_spin.value() as u32;
        let accuracy = match self_.accuracy_combo.selected() {
            0 => Accuracy::None,
            1 => Accuracy::Country,
            2 => Accuracy::City,
            3 => Accuracy::Neighborhood,
            4 => Accuracy::Street,
            5 => Accuracy::Exact,
            _ => unimplemented!(),
        };
        let root = self.native().unwrap();

        ctx.spawn_local(clone!(@weak self as page => async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            if let Ok(location) = locate(&identifier, distance_threshold, time_threshold, accuracy).await {
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
    identifier: &WindowIdentifier,
    distance_threshold: u32,
    time_threshold: u32,
    accuracy: Accuracy,
) -> ashpd::Result<Location> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = LocationProxy::new(&connection).await?;
    let session = proxy
        .create_session(
            Some(distance_threshold),
            Some(time_threshold),
            Some(accuracy),
        )
        .await?;

    let (_, location) = futures::try_join!(
        proxy.start(&session, identifier).into_future(),
        proxy.receive_location_updated().into_future()
    )?;

    session.close().await?;
    Ok(location)
}
