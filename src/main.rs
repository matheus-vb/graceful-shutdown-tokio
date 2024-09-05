mod lib;
use std::{fmt, time::Duration};

use futures_util::{future::BoxFuture, FutureExt};
use lib::{graceful::GracefulShutdown, subsys_tree::SubsystemNode};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle, Toplevel};

#[tokio::main]
async fn main() {
    let mut root = SubsystemNode::new_root(do_thing_generator(), Duration::from_secs(1));
    let _child = root.new_child(do_longer_thing_generator(), Duration::from_secs(2));

    tokio::time::sleep(Duration::from_secs(5)).await;

    root.shutdown().await;
}

fn do_thing_generator() -> impl Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static {
    || do_thing().boxed()
}

fn do_longer_thing_generator() -> impl Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static {
    || do_longer_thing().boxed()
}

async fn do_thing() {
    println!("Doing thing");
    tokio::time::sleep(Duration::from_secs(20)).await;
}

async fn do_longer_thing() {
    println!("Doing longer thing");
    tokio::time::sleep(Duration::from_secs(2)).await;
}

async fn run_dependency() {
    loop {
        todo!()
    }
}
