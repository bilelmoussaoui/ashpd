use std::sync::LazyLock;

pub static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

pub fn spawn_tokio_blocking<F>(fut: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();

    RUNTIME.spawn(async {
        let response = fut.await;
        sender.send(response)
    });
    receiver.blocking_recv().unwrap()
}

pub async fn spawn_tokio<F>(fut: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();

    RUNTIME.spawn(async {
        let response = fut.await;
        sender.send(response)
    });
    receiver.await.unwrap()
}

pub fn bridge_stream<S, St, T, E>(
    stream_future: S,
) -> tokio::sync::mpsc::UnboundedReceiver<Result<T, E>>
where
    S: std::future::Future<Output = Result<St, E>> + Send + 'static,
    St: futures_util::Stream<Item = T> + Send + Unpin + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    use futures_util::StreamExt;

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    RUNTIME.spawn(async move {
        match stream_future.await {
            Ok(mut stream) => {
                while let Some(item) = stream.next().await {
                    if sender.send(Ok(item)).is_err() {
                        break;
                    }
                }
            }
            Err(e) => {
                let _ = sender.send(Err(e));
            }
        }
    });

    receiver
}

pub fn is_empty(txt: gtk::glib::GString) -> Option<String> {
    if txt.is_empty() {
        None
    } else {
        Some(txt.to_string())
    }
}

pub fn split_comma(txt: String) -> Vec<String> {
    txt.split(',')
        .filter(|e| !e.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
}

pub mod desktop;
mod documents;

pub use documents::DocumentsPage;
