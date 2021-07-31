use ashpd::{desktop::open_uri, WindowIdentifier};
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/open_uri.ui")]
    pub struct OpenUriPage {
        #[template_child]
        pub writeable_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub ask_switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OpenUriPage {
        const NAME: &'static str = "OpenUriPage";
        type Type = super::OpenUriPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<adw::ClampLayout>();
            klass.install_action("open_uri.uri", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.open_uri().await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for OpenUriPage {}
    impl WidgetImpl for OpenUriPage {}
    impl BinImpl for OpenUriPage {}
}

glib::wrapper! {
    pub struct OpenUriPage(ObjectSubclass<imp::OpenUriPage>) @extends gtk::Widget, adw::Bin;
}

impl OpenUriPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a OpenUriPage")
    }

    async fn open_uri(&self) {
        let self_ = imp::OpenUriPage::from_instance(self);
        let writable = self_.writeable_switch.is_active();
        let ask = self_.ask_switch.is_active();
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let _ = open_uri::open_uri(
            &identifier,
            "https://github.com/bilelmoussaoui/ashpd",
            writable,
            ask,
        )
        .await;
    }
}
