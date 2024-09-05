use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use futures_util::future::BoxFuture;
use futures_util::{Future, FutureExt};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait TreeLogic: Send + Sync + 'static {
    fn run(&self);
}

pub struct SubsystemNode {
    token: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
    children: Vec<Arc<Mutex<SubsystemNode>>>,
}

impl SubsystemNode {
    pub fn new_root<F>(fut: F, interval: Duration) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let token = CancellationToken::new();
        let task_handle = Some(Self::spawn_task(token.clone(), fut, interval));

        SubsystemNode {
            token,
            task_handle,
            children: vec![],
        }
    }

    pub fn new_child<F>(&mut self, fut: F, interval: Duration) -> Arc<Mutex<Self>>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let token = self.token.child_token();
        let task_handle = Some(Self::spawn_task(token.clone(), fut, interval));

        let child = Arc::new(Mutex::new(SubsystemNode {
            token,
            task_handle,
            children: vec![],
        }));

        self.add_child(child.clone());
        child
    }

    fn add_child(&mut self, child: Arc<Mutex<SubsystemNode>>) {
        self.children.push(child);
    }

    fn spawn_task<F>(token: CancellationToken, fut: F, interval: Duration) -> JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(async move {
            tokio::select! {
                _ = fut => {
                    println!("Task done");
                }
                _ = token.cancelled() => {
                    println!("Shutting down with token");
                    return;
                }
            }
        })
    }

    pub fn shutdown(&mut self) -> BoxFuture<'_, ()> {
        self.token.cancel();

        async move {
            for child in &self.children {
                let mut child_lock = child.lock().await;
                child_lock.shutdown().await;
            }

            if let Some(handle) = self.task_handle.take() {
                let _ = handle.await;
            }
        }
        .boxed()
    }
}
