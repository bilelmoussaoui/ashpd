use std::{convert::TryFrom, sync::Arc};

use ashpd::zbus;
use ashpd::{
    desktop::screencast::{
        AsyncScreenCastProxy, CreateSession, CreateSessionOptions, SelectSourcesOptions,
        SourceType, StartCastOptions, Streams,
    },
    BasicResponse, HandleToken,
};
use ashpd::{AsyncSessionProxy, Response, WindowIdentifier};
use futures::lock::Mutex;
use futures::FutureExt;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::widgets::CameraPaintable;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screencast.ui")]
    pub struct ScreenCastPage {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        pub paintable: CameraPaintable,
    }

    impl Default for ScreenCastPage {
        fn default() -> Self {
            Self {
                picture: TemplateChild::default(),
                response_group: TemplateChild::default(),
                paintable: CameraPaintable::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenCastPage {
        const NAME: &'static str = "ScreenCastPage";
        type Type = super::ScreenCastPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("screencast.start", None, move |page, _action, _target| {
                page.screencast();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenCastPage {
        fn constructed(&self, _obj: &Self::Type) {
            self.picture.set_paintable(Some(&self.paintable));
        }
    }
    impl WidgetImpl for ScreenCastPage {}
    impl BinImpl for ScreenCastPage {}
}

glib::wrapper! {
    pub struct ScreenCastPage(ObjectSubclass<imp::ScreenCastPage>) @extends gtk::Widget, adw::Bin;
}

impl ScreenCastPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a ScreenCastPage")
    }

    pub fn screencast(&self) {
        let self_ = imp::ScreenCastPage::from_instance(self);

        let ctx = glib::MainContext::default();
        let paintable = &self_.paintable;
        let response_group = self_.response_group.get();
        ctx.spawn_local(clone!(@weak paintable, @weak response_group => async move {
            if let Ok(Response::Ok(streams)) = screencast().await {
                let stream = streams.streams().get(0).unwrap();
                println!("{:#?}", stream);
                println!("{:#?}", stream.pipewire_node_id());
                response_group.show();
                paintable.set_pipewire_fd(stream.pipewire_node_id());
            }
        }));
    }
}

pub async fn screencast() -> zbus::Result<Response<Streams>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncScreenCastProxy::new(&connection)?;
    let request = proxy
        .create_session(
            CreateSessionOptions::default()
                .session_handle_token(HandleToken::try_from("handletoken").unwrap()),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<CreateSession>| {
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
    let create_session = receiver.await.unwrap();
    if let Response::Err(err) = create_session {
        return Ok(Response::Err(err));
    }
    let create_session = create_session.unwrap();
    println!("{:#?}", create_session);

    let session = AsyncSessionProxy::new_for_path(&connection, create_session.session_handle())?;
    let request = proxy
        .select_sources(
            &session,
            SelectSourcesOptions::default()
                .multiple(true)
                .types(SourceType::Window | SourceType::Monitor),
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

    if let Response::Err(err) = receiver.await.unwrap() {
        return Ok(Response::Err(err));
    }

    let request = proxy
        .start(
            &session,
            WindowIdentifier::default(),
            StartCastOptions::default(),
        )
        .await?;
    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<Streams>| {
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
