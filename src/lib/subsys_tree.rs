use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use futures_util::future::BoxFuture;
use futures_util::{Future, FutureExt};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct SubsystemNode {
    notify: Arc<Notify>,
    task_handle: Option<JoinHandle<()>>,
    children: Vec<Arc<Mutex<SubsystemNode>>>,
}

impl SubsystemNode {
    pub fn new_root<F>(fut: F, interval: Duration) -> Self
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let notify = Arc::new(Notify::new());
        let task_handle = Some(Self::spawn_task(notify.clone(), fut, interval));

        SubsystemNode {
            notify,
            task_handle,
            children: vec![],
        }
    }

    pub fn new_child<F>(&mut self, fut: F, interval: Duration) -> Arc<Mutex<Self>>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let notify = Arc::new(Notify::new());
        let task_handle = Some(Self::spawn_task(notify.clone(), fut, interval));

        let child = Arc::new(Mutex::new(SubsystemNode {
            notify,
            task_handle,
            children: vec![],
        }));

        self.add_child(child.clone());
        child
    }

    fn add_child(&mut self, child: Arc<Mutex<SubsystemNode>>) {
        self.children.push(child);
    }

    fn spawn_task<F>(notify: Arc<Notify>, fut_gen: F, interval: Duration) -> JoinHandle<()>
    where
        F: Fn() -> BoxFuture<'static, ()> + Send + 'static,
    {
        tokio::spawn(async move {
            loop {
                println!("Entering loop");
                let fut = fut_gen();
                fut.await;

                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        println!("Time elapsed");
                    }
                    _ = notify.notified() => {
                        println!("Shutting down with notify for {interval:?}");
                        return;
                    }
                }
            }
        })
    }

    pub fn shutdown(&mut self) -> BoxFuture<'_, ()> {
        println!("Trigger shutdown");

        async move {
            // First, trigger the shutdown by notifying the task
            self.notify.notify_one();

            // Wait for the parent task to complete
            if let Some(handle) = self.task_handle.take() {
                println!("Waiting for parent task to complete...");
                let _ = handle.await;
                println!("Parent task completed");
            }

            // After parent has completed, shutdown all the children
            for child in &self.children {
                let mut child_lock = child.lock().await;
                println!("Shutting down child...");
                child_lock.shutdown().await;
                println!("Child shutdown completed");
            }
        }
        .boxed()
    }
}
