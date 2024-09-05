mod lib;
use std::time::Duration;

use futures_util::future;
use lib::graceful::{self, GracefulShutdown};

#[tokio::main]
async fn main() {
    let shutdown = GracefulShutdown::new();

    let t1 = tokio::spawn(run_thread(shutdown.clone()));
    let t2 = tokio::spawn(run_thread(shutdown.clone()));
    let t3 = tokio::spawn(run_thread(shutdown.clone()));

    future::join3(t1, t2, t3).await;
}

async fn run_thread(shutdown: GracefulShutdown) {
    tokio::select! {
        _ = shutdown.watch() => { println!("Shutdown called."); }
        _ = do_shit() => { println!("Broke loop"); }
    }
}

async fn do_shit() {
    loop {
        println!("Doing shit");

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
