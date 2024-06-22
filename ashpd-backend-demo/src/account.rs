use std::ops::ControlFlow;

use ashpd::{
    backend::{
        account::{AccountImpl, UserInformationOptions},
        request::RequestImpl,
    },
    desktop::{account::UserInformation, Response},
    AppID, ExternalWindow, WindowIdentifierType,
};
use async_trait::async_trait;
use gtk::prelude::*;

use crate::account_preview::AccountPreview;

#[derive(Default)]
pub struct Account;

#[async_trait]
impl RequestImpl for Account {
    async fn close(&self) {
        log::debug!("IN Close()");
    }
}

#[async_trait]
impl AccountImpl for Account {
    async fn get_user_information(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        options: UserInformationOptions,
    ) -> Response<UserInformation> {
        log::debug!("IN GetUserInformation({app_id}, {window_identifier:?}, {options:?})",);
        let flow: ControlFlow<(), UserInformation> = {
            log::debug!("Opening account preview");
            let preview = AccountPreview::default();
            preview.set_heading(&app_id, options.reason());
            {
                let external_window = ExternalWindow::new(window_identifier);
                let fake_window = ExternalWindow::fake(external_window.as_ref());
                preview.set_transient_for(Some(&fake_window));

                if let Some(ref external) = external_window {
                    gtk::Widget::realize(preview.upcast_ref());
                    external.set_parent_of(&preview.surface().unwrap());
                }
            }
            preview.present_and_wait().await
        };

        let response = match flow {
            ControlFlow::Break(()) => Response::cancelled(),
            ControlFlow::Continue(user_info) => Response::Ok(user_info),
        };

        log::debug!("OUT GetUserInformation({response:?})",);
        response
    }
}
