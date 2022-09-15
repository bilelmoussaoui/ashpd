use std::ops::ControlFlow;

use futures_channel::oneshot;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::sync::Mutex;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(file = "wallpaper_preview.ui")]
    pub struct WallpaperPreview {
        pub(super) sender: Mutex<Option<oneshot::Sender<ControlFlow<()>>>>,
        pub(super) receiver: Mutex<Option<oneshot::Receiver<ControlFlow<()>>>>,
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
    }

    impl Default for WallpaperPreview {
        fn default() -> Self {
            let (sender, receiver) = oneshot::channel::<ControlFlow<()>>();
            Self {
                sender: Mutex::new(Some(sender)),
                receiver: Mutex::new(Some(receiver)),
                picture: TemplateChild::default(),
                stack: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperPreview {
        const NAME: &'static str = "WallpaperPreview";
        type Type = super::WallpaperPreview;
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
    impl WallpaperPreview {
        #[template_callback]
        fn on_dialog_cancel(&self) {
            let sender = self.sender.lock().unwrap().take();
            if let Some(sender) = sender {
                let _ = sender.send(ControlFlow::Break(()));
            }
            self.obj().close();
        }
        #[template_callback]
        fn on_dialog_set(&self) {
            let sender = self.sender.lock().unwrap().take();
            if let Some(sender) = sender {
                let _ = sender.send(ControlFlow::Continue(()));
            }
            self.obj().close();
        }
    }

    impl ObjectImpl for WallpaperPreview {}
    impl WidgetImpl for WallpaperPreview {}
    impl WindowImpl for WallpaperPreview {}
}

glib::wrapper! {
    pub struct WallpaperPreview(ObjectSubclass<imp::WallpaperPreview>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Native;
}

impl Default for WallpaperPreview {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl WallpaperPreview {
    pub async fn present_and_wait(self) -> ControlFlow<()> {
        self.present();
        let receiver = self.imp().receiver.lock().unwrap().take();
        if let Some(receiver) = receiver {
            receiver.await.unwrap()
        } else {
            ControlFlow::Break(())
        }
    }

    pub fn set_uri(&self, uri: &url::Url) {
        let imp = self.imp();
        imp.stack.set_visible_child_name("preview-state");
        let file = gio::File::for_uri(uri.as_str());
        imp.picture.set_file(Some(&file));
    }
}

unsafe impl Send for WallpaperPreview {}

unsafe impl Sync for WallpaperPreview {}
