use std::ops::ControlFlow;

use ashpd::desktop::account::UserInformation;
use futures_channel::oneshot;
use gtk::{
    gdk, gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

mod imp {
    use std::sync::Mutex;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(file = "account_preview.ui")]
    pub struct AccountPreview {
        pub(super) sender: Mutex<Option<oneshot::Sender<ControlFlow<(), UserInformation>>>>,
        pub(super) receiver: Mutex<Option<oneshot::Receiver<ControlFlow<(), UserInformation>>>>,
        pub(super) file: Mutex<Option<gio::File>>,
        #[template_child]
        pub heading: TemplateChild<gtk::Label>,
        #[template_child]
        pub avatar: TemplateChild<adw::Avatar>,
        #[template_child]
        pub username: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub display_name: TemplateChild<adw::EntryRow>,
    }

    impl Default for AccountPreview {
        fn default() -> Self {
            let (sender, receiver) = oneshot::channel::<ControlFlow<(), UserInformation>>();
            Self {
                sender: Mutex::new(Some(sender)),
                receiver: Mutex::new(Some(receiver)),
                file: Mutex::new(None),
                heading: TemplateChild::default(),
                avatar: TemplateChild::default(),
                username: TemplateChild::default(),
                display_name: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccountPreview {
        const NAME: &'static str = "AccountPreview";
        type Type = super::AccountPreview;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl AccountPreview {
        #[template_callback]
        fn on_dialog_cancel(&self) {
            let sender = self.sender.lock().unwrap().take();
            if let Some(sender) = sender {
                let _ = sender.send(ControlFlow::Break(()));
            }
            self.obj().close();
        }
        #[template_callback]
        fn on_dialog_ok(&self) {
            let sender = self.sender.lock().unwrap().take();
            if let Some(sender) = sender {
                let uri =
                    url::Url::parse(&self.file.lock().unwrap().as_ref().unwrap().uri()).unwrap();
                let _ = sender.send(ControlFlow::Continue(UserInformation::new(
                    &self.username.text(),
                    &self.display_name.text(),
                    uri,
                )));
            }
            self.obj().close();
        }
        #[template_callback]
        async fn image_button_clicked(&self) {
            if let Ok(file) = gtk::FileDialog::builder()
                .title("Select an image")
                .build()
                .open_future(Some(&*self.obj()))
                .await
            {
                self.obj().set_avatar_file(&file);
            }
        }
    }

    impl ObjectImpl for AccountPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let ctx = glib::MainContext::default();
            let preview = self.obj();
            ctx.spawn_local(clone!(@strong preview => async move {
                if let Ok(user_info) = UserInformation::current_user().await {
                    let file = gio::File::for_uri(user_info.image().as_str());
                    preview.set_avatar_file(&file);

                    preview.imp().display_name
                        .set_text(user_info.name());
                    preview.imp().username
                        .set_text(user_info.id());
                }
            }));
        }
    }
    impl WidgetImpl for AccountPreview {}
    impl WindowImpl for AccountPreview {}
}

glib::wrapper! {
    pub struct AccountPreview(ObjectSubclass<imp::AccountPreview>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Native;
}

impl Default for AccountPreview {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl AccountPreview {
    pub async fn present_and_wait(self) -> ControlFlow<(), UserInformation> {
        self.present();
        let receiver = self.imp().receiver.lock().unwrap().take();
        if let Some(receiver) = receiver {
            receiver.await.unwrap()
        } else {
            ControlFlow::Break(())
        }
    }

    fn set_avatar_file(&self, file: &gio::File) {
        let imp = self.imp();

        match gdk::Texture::from_file(file) {
            Ok(texture) => {
                imp.avatar.set_custom_image(Some(&texture));
                imp.file.lock().unwrap().replace(file.clone());
                imp.avatar.remove_css_class("dim-label");
            }
            Err(err) => {
                log::error!("Failed to set avatar from file {err}");
                imp.avatar.set_custom_image(gdk::Paintable::NONE);
                imp.avatar.set_icon_name(Some("camera-photo-symbolic"));
                imp.avatar.add_css_class("dim-label");
            }
        }
    }

    pub fn set_heading(&self, app_id: &ashpd::AppID, reason: &str) {
        let heading = if !app_id.is_empty() {
            let desktop_file = format!("{app_id}.desktop");
            let app_info = gio::DesktopAppInfo::new(&desktop_file).unwrap();
            let display_name = app_info.display_name();
            format!(
                "Share your personal information with {} {}",
                display_name, reason
            )
        } else {
            format!(
                "Share your personal information with the requesting application? {}",
                reason
            )
        };
        self.imp().heading.set_text(&heading);
    }
}

unsafe impl Send for AccountPreview {}

unsafe impl Sync for AccountPreview {}
