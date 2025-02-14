use std::{path::PathBuf, sync::OnceLock};

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    desktop::{
        dynamic_launcher::{
            DynamicLauncherProxy, LauncherIcon, LauncherType, PrepareInstallOptions,
        },
        Icon,
    },
    WindowIdentifier,
};
use gtk::{
    gdk, gio,
    glib::{self, clone},
};

use crate::{
    config,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod desktop_file_row {
    use super::*;

    mod imp {
        use std::cell::OnceCell;

        use ashpd::desktop::dynamic_launcher::IconType;

        use super::*;

        #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
        #[template(resource = "/com/belmoussaoui/ashpd/demo/dynamic_launcher_row.ui")]
        #[properties(wrapper_type = super::DesktopFileRow)]
        pub struct DesktopFileRow {
            #[property(get, construct_only)]
            desktop_id: OnceCell<String>,
            pub(super) entry: OnceCell<String>,
            pub(super) launcher_icon: OnceCell<LauncherIcon>,
            #[template_child]
            pub(super) text_view: TemplateChild<gtk::TextView>,
            #[template_child]
            pub(super) icon: TemplateChild<gtk::Image>,
            #[template_child]
            pub(super) format_row: TemplateChild<adw::ActionRow>,
            #[template_child]
            pub(super) size_row: TemplateChild<adw::ActionRow>,
            #[template_child]
            uninstall_button: TemplateChild<adw::ButtonRow>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for DesktopFileRow {
            const NAME: &'static str = "DesktopFileRow";
            type Type = super::DesktopFileRow;
            type ParentType = gtk::ListBoxRow;

            fn class_init(klass: &mut Self::Class) {
                klass.bind_template();
            }

            fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
                obj.init_template();
            }
        }

        #[glib::derived_properties]
        impl ObjectImpl for DesktopFileRow {
            fn constructed(&self) {
                self.parent_constructed();
                let obj = self.obj().to_owned();
                self.uninstall_button
                    .set_action_target(Some(obj.desktop_id().to_variant()));

                glib::spawn_future_local(async move {
                    let desktop_id = obj.desktop_id();
                    if let Err(err) = obj.load_desktop_file(&desktop_id).await {
                        tracing::error!("Failed to load desktop file {err}");
                        return;
                    }

                    let imp = obj.imp();
                    let buffer = imp.text_view.buffer();
                    let mut pos = buffer.start_iter();
                    buffer.insert(&mut pos, imp.entry.get().unwrap());

                    let icon = imp.launcher_icon.get().unwrap();
                    let Icon::Bytes(bytes) = icon.icon() else {
                        return;
                    };
                    let bytes = glib::Bytes::from_owned(bytes);
                    let paintable = gdk::Texture::from_bytes(&bytes).unwrap();

                    let format = match icon.type_() {
                        IconType::Svg => "SVG",
                        IconType::Png => "PNG",
                        IconType::Jpeg => "JPEG",
                    };
                    imp.format_row.set_subtitle(format);
                    imp.size_row.set_subtitle(&icon.size().to_string());

                    imp.icon.set_pixel_size(icon.size().clamp(32, 96) as i32);
                    imp.icon.set_paintable(Some(&paintable));
                });
            }
        }
        impl WidgetImpl for DesktopFileRow {}
        impl ListBoxRowImpl for DesktopFileRow {}
    }

    glib::wrapper! {
        pub struct DesktopFileRow(ObjectSubclass<imp::DesktopFileRow>)
            @extends gtk::Widget, gtk::ListBoxRow;
    }

    impl DesktopFileRow {
        pub fn new(desktop_id: &str) -> Self {
            glib::Object::builder()
                .property("desktop-id", desktop_id)
                .build()
        }

        async fn load_desktop_file(&self, desktop_id: &str) -> ashpd::Result<()> {
            let imp = self.imp();
            let proxy = DynamicLauncherProxy::new().await?;
            let entry = proxy.desktop_entry(desktop_id).await?;
            let launcher_icon = proxy.icon(desktop_id).await?;
            imp.entry.set(entry).unwrap();
            imp.launcher_icon.set(launcher_icon).unwrap();
            Ok(())
        }
    }
}

// Used for caching the installed apps
static DEFAULT_DESKTOP_FILE: &str = r#"
[Desktop Entry]
Name=ASHPD Demo
Exec=xdg-open https://github.com
Type=Application"#;
mod imp {
    use std::cell::RefCell;

    use super::{desktop_file_row::DesktopFileRow, *};

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/dynamic_launcher.ui")]
    pub struct DynamicLauncherPage {
        #[template_child]
        pub(super) name_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) icon_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) modal_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) editable_name_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) editable_icon_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(super) launcher_type_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) launcher_target_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) desktop_file_id_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) desktop_file_content: TemplateChild<gtk::TextView>,
        #[template_child]
        pub(super) response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub(super) token_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) icon: TemplateChild<gtk::Image>,

        #[template_child]
        pub(super) installed_apps: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) installed_apps_label: TemplateChild<gtk::Label>,

        pub(super) installed_apps_cache: gtk::StringList,
        pub(super) selected_icon: RefCell<Option<gio::File>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DynamicLauncherPage {
        const NAME: &'static str = "DynamicLauncherPage";
        type Type = super::DynamicLauncherPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action_async(
                "dynamic_launcher.install",
                None,
                |widget, _, _| async move {
                    match widget.install().await {
                        Ok(_) => {
                            widget.success("Launcher installed");
                            if let Err(err) = widget.save_cache().await {
                                tracing::error!("Failed to cache installed apps {err}");
                            }
                        }
                        Err(ashpd::Error::UnexpectedIcon) => {
                            widget.error("Launcher icon is required");
                        }
                        Err(err) => {
                            widget.error("Failed to install launcher");
                            tracing::error!("Failed to install launcher {err}");
                        }
                    }
                },
            );
            klass.install_action_async(
                "dynamic_launcher.uninstall",
                Some(&*str::static_variant_type()),
                |widget, _, target| async move {
                    let desktop_id = target.unwrap();
                    match widget.uninstall(desktop_id.str().unwrap()).await {
                        Ok(_) => {
                            widget.success("Launcher uninstalled");
                            if let Err(err) = widget.save_cache().await {
                                tracing::error!("Failed to cache installed apps {err}");
                            }
                        }
                        Err(err) => {
                            widget.error("Failed to uninstall launcher");
                            tracing::error!("Failed to uninstall launcher {err}");
                        }
                    }
                },
            );
            klass.install_action_async(
                "dynamic_launcher.select-icon",
                None,
                |widget, _, _| async move {
                    if let Err(err) = widget.select_icon().await {
                        widget.error("Failed to select launcher icon");
                        tracing::error!("Failed to select launcher icon {err}");
                    }
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for DynamicLauncherPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            glib::spawn_future_local(clone!(
                #[weak]
                obj,
                async move {
                    if let Err(err) = obj.load_cache().await {
                        tracing::error!("Failed to load cache {err}");
                    }
                }
            ));

            let buffer = self.desktop_file_content.buffer();
            let mut pos = buffer.start_iter();
            buffer.insert(&mut pos, DEFAULT_DESKTOP_FILE.trim());

            self.installed_apps
                .bind_model(Some(&self.installed_apps_cache), |obj| {
                    let desktop_file = obj.downcast_ref::<gtk::StringObject>().unwrap().string();

                    DesktopFileRow::new(&desktop_file).upcast()
                });
        }
    }
    impl WidgetImpl for DynamicLauncherPage {}
    impl BinImpl for DynamicLauncherPage {}
    impl PortalPageImpl for DynamicLauncherPage {}
}

glib::wrapper! {
    pub struct DynamicLauncherPage(ObjectSubclass<imp::DynamicLauncherPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl DynamicLauncherPage {
    fn launcher_type(&self) -> LauncherType {
        let imp = self.imp();
        match imp.launcher_type_row.selected() {
            0 => LauncherType::Application,
            _ => LauncherType::WebApplication,
        }
    }

    fn cache_file() -> &'static PathBuf {
        static CACHE_FILE: OnceLock<PathBuf> = OnceLock::new();
        CACHE_FILE.get_or_init(|| glib::user_data_dir().join("installed-apps.txt"))
    }
    async fn save_cache(&self) -> Result<(), glib::Error> {
        let file = gio::File::for_path(Self::cache_file());
        let installed_apps_cache = &self.imp().installed_apps_cache;

        let mut data = String::new();
        let mut index = 0;
        while index < installed_apps_cache.n_items() {
            data.push_str(&format!(
                "{}\n",
                installed_apps_cache.string(index).unwrap()
            ));

            index += 1;
        }
        file.replace_contents_future(data, None, false, gio::FileCreateFlags::REPLACE_DESTINATION)
            .await
            .map_err(|e| e.1)?;
        Ok(())
    }

    async fn load_cache(&self) -> Result<(), glib::Error> {
        let imp = self.imp();
        let file = gio::File::for_path(Self::cache_file());
        let (buffer, _) = file.load_contents_future().await?;
        let lines = std::str::from_utf8(&buffer).expect("Valid utf8").lines();
        for desktop_id in lines {
            if desktop_id.is_empty() || !desktop_id.starts_with(config::APP_ID) {
                continue;
            }
            imp.installed_apps_cache.append(desktop_id);
        }

        let has_items = imp.installed_apps_cache.n_items() != 0;
        imp.installed_apps.set_visible(has_items);
        imp.installed_apps_label.set_visible(has_items);
        Ok(())
    }

    async fn select_icon(&self) -> Result<(), glib::Error> {
        let imp = self.imp();
        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let root = self.native().unwrap();
        let file = gtk::FileDialog::builder()
            .accept_label("Select")
            .modal(true)
            .title("Launcher Icon")
            .filters(&filters)
            .build()
            .open_future(root.downcast_ref::<gtk::Window>())
            .await?;

        imp.icon_row.set_subtitle(&file.uri());
        imp.selected_icon.replace(Some(file));
        Ok(())
    }

    async fn install(&self) -> ashpd::Result<()> {
        let imp = self.imp();
        if imp.selected_icon.borrow().is_none() {
            self.error("An icon is required");
            return Err(ashpd::Error::UnexpectedIcon);
        }

        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let proxy = DynamicLauncherProxy::new().await?;

        let launcher_name = imp.name_row.text();
        let modal = imp.modal_switch.is_active();
        let editable_name = imp.editable_name_switch.is_active();
        let editable_icon = imp.editable_icon_switch.is_active();
        let launcher_type = self.launcher_type();
        let desktop_id = format!(
            "{}.{}",
            crate::config::APP_ID,
            imp.desktop_file_id_row.text()
        );
        let mut options = PrepareInstallOptions::default()
            .modal(modal)
            .editable_icon(editable_icon)
            .editable_name(editable_name)
            .launcher_type(launcher_type);
        if launcher_type == LauncherType::WebApplication {
            let target = imp.launcher_target_row.text();
            options = options.target(&*target);
        }

        let selected_file = imp.selected_icon.borrow().clone();
        let (data, _) = selected_file
            .as_ref()
            .expect("An icon is required")
            .load_contents_future()
            .await
            .unwrap();
        let icon = Icon::Bytes(data.to_vec());

        let response = proxy
            .prepare_install(identifier.as_ref(), &launcher_name, icon, options)
            .await?
            .response()?;

        imp.response_group.set_visible(true);
        imp.token_label.set_text(response.token());
        imp.name_label.set_text(response.name());
        let Icon::Bytes(bytes) = response.icon() else {
            unreachable!();
        };
        let paintable = gdk::Texture::from_bytes(&glib::Bytes::from_owned(bytes))
            .expect("Failed to create texture");
        imp.icon.set_paintable(Some(&paintable));

        let (start_iter, end_iter) = imp.desktop_file_content.buffer().bounds();
        let desktop_entry = imp
            .desktop_file_content
            .buffer()
            .text(&start_iter, &end_iter, true);

        proxy
            .install(response.token(), &desktop_id, &desktop_entry)
            .await?;

        imp.installed_apps_cache.append(&desktop_id);
        let has_items = imp.installed_apps_cache.n_items() != 0;
        imp.installed_apps.set_visible(has_items);
        imp.installed_apps_label.set_visible(has_items);
        Ok(())
    }

    async fn uninstall(&self, desktop_id: &str) -> ashpd::Result<()> {
        let imp = self.imp();
        let proxy = DynamicLauncherProxy::new().await?;
        proxy.uninstall(desktop_id).await?;

        let model = &imp.installed_apps_cache;
        let mut index = 0;
        while index < model.n_items() {
            let item = model.string(index).unwrap();
            if item == desktop_id {
                model.remove(index);
                break;
            }
            index += 1;
        }
        let has_items = model.n_items() != 0;
        imp.installed_apps.set_visible(has_items);
        imp.installed_apps_label.set_visible(has_items);
        Ok(())
    }
}
