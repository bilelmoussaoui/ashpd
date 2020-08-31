use libportal::desktop::account::{AccountProxy, UserInfoOptionsBuilder};
use libportal::desktop::screenshot::*;
use libportal::zbus;
use libportal::RequestProxy;
use libportal::WindowIdentifier;
/*
use libportal::desktop::*;
use libportal::zbus;
use libportal::zvariant::Value;

fn main() -> zbus::fdo::Result<()> {
    let connection = zbus::Connection::new_session()?;
    let proxy = ScreenshotProxy::new(&connection)?;

    //let request = proxy.pick_color("test", PickColorOptions::default())?;
    println!("{:#?}", PickColorOptions::default());

    Ok(())
}

fn main() -> zbus::fdo::Result<()> {
    let connection = zbus::Connection::new_session()?;
    /*
    let proxy = ScreenshotProxy::new(&connection)?;
    let request_handle =
        proxy.pick_color(WindowIdentifier::default(), PickColorOptions::default())?;
    */
    let proxy = AccountProxy::new(&connection)?;
    let request_handle = proxy.get_user_information(
        WindowIdentifier::default(),
        UserInfoOptionsBuilder::new("Fractal would like access to your files").build(),
    )?;
    // let req = RequestProxy::new(&connection, &request_handle)?;
    //req.close()?;
    Ok(())
}

*/


fn main() -> zbus::fdo::Result<()> {
    let connection = zbus::Connection::new_session()?;
    let proxy = ScreenshotProxy::new(&connection)?;
    let request_handle = proxy.screenshot(
        WindowIdentifier::default(),
        ScreenshotOptionsBuilder::default()
            .interactive(false)
            .build()
    )?;
    let req = RequestProxy::new(&connection, &request_handle)?;


    //req.close()?;
    Ok(())
}
