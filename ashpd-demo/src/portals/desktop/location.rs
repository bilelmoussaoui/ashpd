use std::sync::Arc;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    desktop::{
        location::{Accuracy, Location, LocationProxy},
        Session,
    },
    WindowIdentifier,
};
use chrono::{DateTime, Local, TimeZone};
use futures::{
    future::{AbortHandle, Abortable},
    lock::Mutex,
};
use gtk::glib::{self, clone};
use shumate::prelude::*;

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
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
        #[template_child]
        pub map: TemplateChild<shumate::Map>,
        #[template_child(id = "license")]
        pub map_license: TemplateChild<shumate::License>,
        pub marker: shumate::Marker,
        pub session: Arc<Mutex<Option<Session<'static>>>>,
        pub abort_handle: Arc<Mutex<Option<AbortHandle>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocationPage {
        const NAME: &'static str = "LocationPage";
        type Type = super::LocationPage;
        type ParentType = PortalPage;

        fn new() -> Self {
            let marker = shumate::Marker::new();
            let marker_img = gtk::Image::from_icon_name("map-marker-symbolic");
            marker_img.add_css_class("map-marker");
            marker.set_child(Some(&marker_img));

            Self {
                marker,
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("location.start", None, |page, _, _| async move {
                page.locate().await;
            });

            klass.install_action_async("location.stop", None, |page, _, _| async move {
                page.stop_session().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for LocationPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            let registry = shumate::MapSourceRegistry::with_defaults();
            let source = registry.by_id(shumate::MAP_SOURCE_OSM_MAPNIK).unwrap();
            obj.action_set_enabled("location.stop", false);
            obj.action_set_enabled("location.start", true);

            let viewport = self.map.viewport().unwrap();

            let layer = shumate::MapLayer::new(&source, &viewport);
            self.map.add_layer(&layer);

            let marker_layer = shumate::MarkerLayer::new(&viewport);
            marker_layer.add_marker(&self.marker);
            self.map.add_layer(&marker_layer);

            self.map.set_map_source(&source);
            viewport.set_reference_map_source(Some(&source));
            viewport.set_zoom_level(6.0);

            // self.map_license.append_map_source(&source);
        }

        fn dispose(&self) {
            let obj = self.obj();
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak obj as page => async move {
                page.stop_session().await;
            }));
        }
    }
    impl WidgetImpl for LocationPage {}
    impl BinImpl for LocationPage {}
    impl PortalPageImpl for LocationPage {}
}

glib::wrapper! {
    pub struct LocationPage(ObjectSubclass<imp::LocationPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl LocationPage {
    async fn locate(&self) {
        let imp = self.imp();
        let distance_threshold = imp.distance_spin.value() as u32;
        let time_threshold = imp.time_spin.value() as u32;
        let accuracy = match imp.accuracy_combo.selected() {
            0 => Accuracy::None,
            1 => Accuracy::Country,
            2 => Accuracy::City,
            3 => Accuracy::Neighborhood,
            4 => Accuracy::Street,
            5 => Accuracy::Exact,
            _ => unimplemented!(),
        };
        let root = self.native().unwrap();

        let identifier = WindowIdentifier::from_native(&root).await;
        match locate(&identifier, distance_threshold, time_threshold, accuracy).await {
            Ok((session, location_proxy)) => {
                imp.session.lock().await.replace(session);
                self.action_set_enabled("location.stop", true);
                self.action_set_enabled("location.start", false);
                loop {
                    if imp.session.lock().await.is_none() {
                        break;
                    }

                    let (abort_handle, abort_registration) = AbortHandle::new_pair();
                    let future = Abortable::new(
                        async {
                            if let Ok(location) = location_proxy.receive_location_updated().await {
                                self.on_location_updated(location);
                            }
                        },
                        abort_registration,
                    );
                    imp.abort_handle.lock().await.replace(abort_handle);
                    let _ = future.await;
                }
            }
            Err(_err) => {
                self.action_set_enabled("location.stop", false);
                self.action_set_enabled("location.start", true);
                self.send_notification("Failed to locate", NotificationKind::Error);
            }
        }
    }

    async fn stop_session(&self) {
        let mut session_lock = self.imp().session.lock().await;
        self.action_set_enabled("location.stop", false);
        self.action_set_enabled("location.start", true);
        if let Some(abort_handle) = self.imp().abort_handle.lock().await.take() {
            abort_handle.abort();
        }
        if let Some(session) = session_lock.take() {
            let _ = session.close().await;
        }
    }

    fn on_location_updated(&self, location: Location) {
        let imp = self.imp();
        imp.response_group.set_visible(true);
        imp.accuracy_label
            .set_label(&location.accuracy().to_string());
        if let Some(altitude) = location.altitude() {
            imp.altitude_label.set_label(&altitude.to_string());
        }
        if let Some(speed) = location.speed() {
            imp.speed_label.set_label(&speed.to_string());
        }
        if let Some(heading) = location.heading() {
            imp.heading_label.set_label(&heading.to_string());
        }
        if let Some(description) = location.description() {
            imp.description_label.set_label(description);
        }
        imp.latitude_label
            .set_label(&location.latitude().to_string());
        imp.longitude_label
            .set_label(&location.longitude().to_string());

        let datetime: DateTime<Local> = Local
            .timestamp_opt(location.timestamp().as_secs() as i64, 0)
            .unwrap();
        let since = datetime.format("%Y-%m-%d %H:%M:%S");
        imp.timestamp_label.set_label(&since.to_string());

        imp.map.center_on(location.latitude(), location.longitude());
        imp.marker
            .set_location(location.latitude(), location.longitude());
        self.send_notification("Position updated", NotificationKind::Success);
    }
}

pub async fn locate<'a>(
    identifier: &WindowIdentifier,
    distance_threshold: u32,
    time_threshold: u32,
    accuracy: Accuracy,
) -> ashpd::Result<(Session<'a>, LocationProxy<'a>)> {
    let proxy = LocationProxy::new().await?;
    let session = proxy
        .create_session(
            Some(distance_threshold),
            Some(time_threshold),
            Some(accuracy),
        )
        .await?;
    proxy.start(&session, identifier).await?;
    Ok((session, proxy))
}
