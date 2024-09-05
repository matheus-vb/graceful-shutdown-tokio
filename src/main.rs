mod lib;
use std::{fmt, time::Duration};

use lib::{graceful::GracefulShutdown, subsys_tree::SubsystemNode};
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle, Toplevel};

#[tokio::main]
async fn main() {
    let mut root = SubsystemNode::new_root(do_thing(), Duration::from_secs(5));
    let _child = root.new_child(do_thing(), Duration::from_secs(5));

    tokio::time::sleep(Duration::from_secs(8)).await;

    root.shutdown().await;
}

async fn do_thing() {
    loop {
        println!("Doing thing");

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_dependency() {
    loop {
        todo!()
    }
}
