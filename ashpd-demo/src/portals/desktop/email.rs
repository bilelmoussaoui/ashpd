use std::sync::Arc;

use ashpd::desktop::email::{AsyncEmailProxy, EmailOptions};
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
    #[template(resource = "/com/belmoussaoui/ashpd/demo/email.ui")]
    pub struct EmailPage {
        #[template_child]
        pub subject: TemplateChild<gtk::Entry>,
        #[template_child]
        pub body: TemplateChild<gtk::Entry>,
        #[template_child]
        pub addresses: TemplateChild<gtk::Entry>,
        #[template_child]
        pub cc_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub bcc_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmailPage {
        const NAME: &'static str = "EmailPage";
        type Type = super::EmailPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("email.compose", None, move |page, _action, _target| {
                page.compose_mail();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for EmailPage {}
    impl WidgetImpl for EmailPage {}
    impl BinImpl for EmailPage {}
}

glib::wrapper! {
    pub struct EmailPage(ObjectSubclass<imp::EmailPage>) @extends gtk::Widget, adw::Bin;
}

impl EmailPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a EmailPage")
    }

    pub fn compose_mail(&self) {
        let self_ = imp::EmailPage::from_instance(self);
        let subject = self_.subject.get_text();
        let body = self_.body.get_text();
        let addresses = self_.addresses.get_text();
        let bcc = self_.bcc_entry.get_text();
        let cc = self_.cc_entry.get_text();
        let root = self.get_root().unwrap();
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let identifier = WindowIdentifier::from_window(&root).await;
            if let Ok(Response::Ok(color)) = compose_email(
                identifier,
                EmailOptions::default()
                    .subject(&subject)
                    .addresses(
                        &addresses
                            .split(",")
                            .filter(|e| e.len() > 1)
                            .collect::<Vec<_>>(),
                    )
                    .bcc(&bcc.split(",").filter(|e| e.len() > 1).collect::<Vec<_>>())
                    .cc(&cc.split(",").filter(|e| e.len() > 1).collect::<Vec<_>>())
                    .body(&body),
            )
            .await
            {
                //TODO: handle the response
                println!("{:#?}", color);
            }
        });
    }
}

pub async fn compose_email(
    window_identifier: WindowIdentifier,
    email: EmailOptions,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncEmailProxy::new(&connection)?;
    let request = proxy.compose_email(window_identifier, email).await?;

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
