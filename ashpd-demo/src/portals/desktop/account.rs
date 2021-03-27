use ashpd::desktop::account::{AccountProxy, UserInfo, UserInfoOptions};
use ashpd::zbus;
use ashpd::{Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/account.ui")]
    pub struct AccountPage {
        #[template_child]
        pub reason: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccountPage {
        const NAME: &'static str = "AccountPage";
        type Type = super::AccountPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "account.information",
                None,
                move |page, _action, _target| {
                    page.get_user_information().unwrap();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for AccountPage {}
    impl WidgetImpl for AccountPage {}
    impl BoxImpl for AccountPage {}
}

glib::wrapper! {
    pub struct AccountPage(ObjectSubclass<imp::AccountPage>) @extends gtk::Widget, gtk::Box;
}

impl AccountPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a AccountPage")
    }

    pub fn get_user_information(&self) -> zbus::fdo::Result<()> {
        let self_ = imp::AccountPage::from_instance(self);
        let buffer = self_.reason.get_buffer();
        let bounds = buffer.get_bounds();
        let reason = buffer.get_text(&bounds.0, &bounds.1, true);
        let options = UserInfoOptions::default().reason(&reason);

        let connection = zbus::Connection::new_session()?;
        let proxy = AccountProxy::new(&connection)?;
        let request = proxy.get_user_information(WindowIdentifier::default(), options)?;
        request.connect_response(|response: Response<UserInfo>| {
            println!("{:#?}", response);
            if let Response::Ok(info) = response {
                println!("{:#?}", info);
            }
            Ok(())
        })?;
        Ok(())
    }
}
