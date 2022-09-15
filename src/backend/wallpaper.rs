use zbus::dbus_interface;
use zvariant::{DeserializeDict, Type};

use crate::{
    desktop::{
        request::{BasicResponse, Response},
        wallpaper::SetOn,
        HandleToken,
    },
    WindowIdentifierType,
};

#[derive(DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct WallpaperOptions {
    #[zvariant(rename = "show-preview")]
    show_preview: Option<bool>,
    #[zvariant(rename = "set-on")]
    set_on: Option<SetOn>,
}

pub struct Wallpaper {}

#[dbus_interface(name = "org.freedesktop.impl.portal.Wallpaper")]
impl Wallpaper {
    async fn set_wallpaper_uri(
        &self,
        handle: HandleToken,
        app_id: &str,
        window_identifier: WindowIdentifierType,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> Response<BasicResponse> {
        todo!()
    }
}
