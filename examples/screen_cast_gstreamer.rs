use ashpd::desktop::PersistMode;
use ashpd::desktop::screencast::Stream;
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use gst::prelude::*;

use std::os::fd::{AsRawFd, OwnedFd};

async fn open_portal() -> ashpd::Result<(Stream, OwnedFd)> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;
    proxy
        .select_sources(
            &session,
            CursorMode::Embedded,
            SourceType::Monitor | SourceType::Window | SourceType::Virtual,
            false,
            None,
            PersistMode::ExplicitlyRevoked,
        )
        .await?;

    let response = proxy.start(&session, None).await?.response()?;
    let stream = response
        .streams()
        .first()
        .expect("No stream found or selected")
        .to_owned();

    let fd = proxy.open_pipe_wire_remote(&session).await?;

    Ok((stream, fd))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gst::init().unwrap();

    let (stream, stream_fd) = open_portal().await?;
    let pipewire_node_id = &stream.pipe_wire_node_id();
    let stream_raw_fd = &stream_fd.as_raw_fd();

    let pipewire_element = gst::ElementFactory::make("pipewiresrc")
        .property("fd", stream_raw_fd)
        .property("path", pipewire_node_id.to_string())
        .build()?;
    let convert = gst::ElementFactory::make("videoconvert").build()?;
    let wayland_sink = gst::ElementFactory::make("waylandsink").build()?;

    let pipeline = gst::Pipeline::default();
    pipeline.add_many([&pipewire_element, &convert, &wayland_sink])?;
    gst::Element::link_many([&pipewire_element, &convert, &wayland_sink])?;

    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline.bus().unwrap();

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                println!("EOS");
                break;
            }
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                eprintln!(
                    "Got error from {}: {} ({})",
                    msg.src()
                        .map(|s| String::from(s.path_string()))
                        .unwrap_or_else(|| "None".into()),
                    err.error(),
                    err.debug().unwrap_or_else(|| "".into()),
                );
                break;
            }
            _ => (),
        }
    }

    Ok(())
}
