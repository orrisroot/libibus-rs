/// Spawn a Tokio task that listens on a D-Bus signal stream.
///
/// The `handler` closure is called for each item yielded by the stream.
/// Returns a [`tokio::task::JoinHandle`] that can be used to abort the task.
pub fn spawn_handler<S, F>(stream: S, mut handler: F) -> tokio::task::JoinHandle<()>
where
    S: futures_util::Stream + Unpin + Send + 'static,
    F: FnMut(S::Item) + Send + 'static,
{
    tokio::spawn(async move {
        use futures_util::StreamExt;
        let mut stream = stream;
        while let Some(item) = stream.next().await {
            handler(item);
        }
    })
}
