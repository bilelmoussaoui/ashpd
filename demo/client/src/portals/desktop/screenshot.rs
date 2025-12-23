use adw::subclass::prelude::*;
use ashpd::{WindowIdentifier, desktop::screenshot};
use gtk::{gdk, gio, glib, prelude::*};

use crate::{
    portals::spawn_tokio,
    widgets::{ColorWidget, PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screenshot.ui")]
    pub struct ScreenshotPage {
        #[template_child]
        pub interactive_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub modal_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub color_widget: TemplateChild<ColorWidget>,
        #[template_child]
        pub screenshot_photo: TemplateChild<gtk::Picture>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotPage {
        const NAME: &'static str = "ScreenshotPage";
        type Type = super::ScreenshotPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("screenshot.pick-color", None, |page, _, _| async move {
                page.pick_color().await;
            });
            klass.install_action_async("screenshot.screenshot", None, |page, _, _| async move {
                page.screenshot().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenshotPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.screenshot_photo.set_overflow(gtk::Overflow::Hidden);
        }
    }
    impl WidgetImpl for ScreenshotPage {}
    impl BinImpl for ScreenshotPage {}
    impl PortalPageImpl for ScreenshotPage {}
}

glib::wrapper! {
    pub struct ScreenshotPage(ObjectSubclass<imp::ScreenshotPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl ScreenshotPage {
    async fn pick_color(&self) {
        // used for retrieving a window identifier
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let response = spawn_tokio(async move {
            screenshot::ColorRequest::default()
                .identifier(identifier)
                .send()
                .await
                .and_then(|r| r.response())
        })
        .await;
        match response {
            Ok(color) => {
                self.imp().color_widget.set_rgba(gdk::RGBA::from(color));
                self.success("Color pick request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to pick color: {err}");
                self.error("Request to pick a color failed");
            }
        }
    }

    async fn screenshot(&self) {
        let imp = self.imp();
        // used for retrieving a window identifier
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let interactive = imp.interactive_switch.is_active();
        let modal = imp.modal_switch.is_active();

        let response = spawn_tokio(async move {
            screenshot::ScreenshotRequest::default()
                .identifier(identifier)
                .interactive(interactive)
                .modal(modal)
                .send()
                .await
                .and_then(|r| r.response())
        })
        .await;
        match response {
            Ok(response) => {
                let file = gio::File::for_uri(response.uri().as_str());
                imp.screenshot_photo.set_file(Some(&file));
                // Revealer has a weird issue where it still
                // takes space even if it's child is hidden
                imp.revealer.set_visible(true);

                imp.revealer.set_reveal_child(true);
                self.success("Screenshot request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to take a screenshot {err}");
                self.error("Request to take a screenshot failed");
            }
        }
    }
}
