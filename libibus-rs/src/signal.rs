/// Spawn a Tokio task that listens on a D-Bus signal stream.
///
/// The `handler` closure is called for each item yielded by the stream.
/// If the handler panics, the task catches the panic and logs it rather
/// than crashing the signal processing pipeline.
///
/// Returns a [`tokio::task::JoinHandle`] that can be used to abort the task.
pub fn spawn_handler<S, F>(stream: S, mut handler: F) -> tokio::task::JoinHandle<()>
where
    S: futures_util::Stream + Unpin + Send + 'static,
    F: FnMut(S::Item) + Send + 'static,
    S::Item: std::panic::UnwindSafe,
{
    tokio::spawn(async move {
        use futures_util::StreamExt;
        let mut stream = stream;
        while let Some(item) = stream.next().await {
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(item))).is_err() {
                log::error!("Signal handler panicked; continuing to listen for signals");
            }
        }
    })
}
