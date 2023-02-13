use ashpd::{desktop::screenshot, WindowIdentifier};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::widgets::{ColorWidget, NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screenshot.ui")]
    pub struct ScreenshotPage {
        #[template_child]
        pub interactive_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub modal_switch: TemplateChild<gtk::Switch>,
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

            klass.install_action_async(
                "screenshot.pick-color",
                None,
                move |page, _action, _target| async move {
                    page.pick_color().await;
                },
            );
            klass.install_action_async(
                "screenshot.screenshot",
                None,
                move |page, _action, _target| async move {
                    page.screenshot().await;
                },
            );
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
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl ScreenshotPage {
    async fn pick_color(&self) {
        // used for retrieving a window identifier
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        match screenshot::Color::builder()
            .identifier(identifier)
            .build()
            .await
            .and_then(|r| r.response())
        {
            Ok(color) => {
                self.imp().color_widget.set_rgba(color.into());
                self.send_notification(
                    "Color pick request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => {
                self.send_notification("Request to pick a color failed", NotificationKind::Error);
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

        match screenshot::ScreenshotRequest::default()
            .identifier(identifier)
            .interactive(interactive)
            .modal(modal)
            .build()
            .await
            .and_then(|r| r.response())
        {
            Ok(response) => {
                let file = gio::File::for_uri(response.uri().as_str());
                imp.screenshot_photo.set_file(Some(&file));
                imp.revealer.show(); // Revealer has a weird issue where it still
                                     // takes space even if it's child is hidden

                imp.revealer.set_reveal_child(true);
                self.send_notification(
                    "Screenshot request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => {
                self.send_notification(
                    "Request to take a screenshot failed",
                    NotificationKind::Error,
                );
            }
        }
    }
}
