use std::sync::Arc;

use tokio::{
    signal::unix::{signal, SignalKind},
    sync::Notify,
};

#[derive(Clone)]
pub struct GracefulShutdown(Arc<GracefulShutdownInner>);

struct GracefulShutdownInner {
    shutdown: Notify,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        let inner = GracefulShutdownInner {
            shutdown: Notify::new(),
        };

        let shutdown = GracefulShutdown(Arc::new(inner));
        shutdown.shutdown_signal(SignalKind::interrupt());
        shutdown.shutdown_signal(SignalKind::terminate());

        shutdown
    }

    pub fn begin(&self) {
        self.0.shutdown.notify_one();
    }

    pub async fn watch(&self) {
        self.0.shutdown.notified().await;
        self.0.shutdown.notify_one();
    }

    fn shutdown_signal(&self, signal_kind: SignalKind) {
        match signal(signal_kind) {
            Ok(mut signal) => {
                let shutdown = self.clone();

                tokio::spawn(async move {
                    if signal.recv().await.is_some() {
                        shutdown.begin();
                    }
                });
            }
            Err(e) => println!("Error listening to signal: {e}"),
        }
    }
}
