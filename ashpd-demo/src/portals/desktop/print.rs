use std::os::unix::prelude::AsRawFd;

use ashpd::{
    desktop::print::{PageSetup, PrintProxy, Settings},
    WindowIdentifier,
};
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/print.ui")]
    pub struct PrintPage {
        #[template_child]
        pub title: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub modal_switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PrintPage {
        const NAME: &'static str = "PrintPage";
        type Type = super::PrintPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async(
                "print.select_file",
                None,
                move |page, _action, _target| async move {
                    page.select_file().await;
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for PrintPage {}
    impl WidgetImpl for PrintPage {}
    impl BinImpl for PrintPage {}
    impl PortalPageImpl for PrintPage {}
}

glib::wrapper! {
    pub struct PrintPage(ObjectSubclass<imp::PrintPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl PrintPage {
    async fn select_file(&self) {
        let imp = self.imp();
        let title = imp.title.text();
        let modal = imp.modal_switch.is_active();
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let file_chooser = gtk::FileChooserNative::builder()
            .accept_label("Select")
            .action(gtk::FileChooserAction::Open)
            .modal(true)
            .transient_for(root.downcast_ref::<gtk::Window>().unwrap())
            .build();
        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("images"));
        file_chooser.add_filter(&filter);

        if file_chooser.run_future().await == gtk::ResponseType::Accept {
            let path = file_chooser.file().unwrap().path().unwrap();
            let file = std::fs::File::open(path).unwrap();

            match print(&identifier, &title, file, modal).await {
                Ok(_) => {
                    self.send_notification(
                        "Print request was successful",
                        NotificationKind::Success,
                    );
                }
                Err(err) => {
                    tracing::error!("Failed to print {}", err);
                    self.send_notification("Request to print failed", NotificationKind::Error);
                }
            }
        };

        file_chooser.destroy();
    }
}

async fn print<F: AsRawFd>(
    identifier: &WindowIdentifier,
    title: &str,
    file: F,
    modal: bool,
) -> ashpd::Result<()> {
    let proxy = PrintProxy::new().await?;

    let out = proxy
        .prepare_print(
            identifier,
            title,
            Settings::default(),
            PageSetup::default(),
            modal,
        )
        .await?;

    proxy
        .print(identifier, title, &file, Some(out.token), modal)
        .await?;

    Ok(())
}
