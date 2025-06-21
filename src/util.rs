use tokio::sync::mpsc;

pub fn create_stub_sender<T>() -> mpsc::UnboundedSender<T> {
    let (tx, _rx) = mpsc::unbounded_channel::<T>();
    tx
}

