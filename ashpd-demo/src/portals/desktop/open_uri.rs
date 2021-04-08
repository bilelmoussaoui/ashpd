use std::sync::Arc;

use ashpd::desktop::open_uri::{AsyncOpenURIProxy, OpenFileOptions};
use ashpd::{zbus, BasicResponse, Response, WindowIdentifier};
use futures::{lock::Mutex, FutureExt};
use gtk::glib;
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
            klass.install_action("open_uri.uri", None, move |page, _action, _target| {
                page.open_uri();
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
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a OpenUriPage")
    }

    pub fn open_uri(&self) {
        let self_ = imp::OpenUriPage::from_instance(self);
        let writable = self_.writeable_switch.get_active();
        let ask = self_.ask_switch.get_active();

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            if let Ok(Response::Ok(color)) = open_uri(
                WindowIdentifier::default(),
                "https://google.com",
                writable,
                ask,
            )
            .await
            {
                //TODO: handle the response
                println!("{:#?}", color);
            }
        });
    }
}

pub async fn open_uri(
    window_identifier: WindowIdentifier,
    uri: &str,
    writable: bool,
    ask: bool,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncOpenURIProxy::new(&connection)?;
    let request = proxy
        .open_uri(
            window_identifier,
            uri,
            OpenFileOptions::default().writeable(writable).ask(ask),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
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
    request.disconnect_signal(signal_id).await?;

    let response = receiver.await.unwrap();
    Ok(response)
}
