extern crate tokio;
extern crate tokio_starvation;

use std::net::SocketAddr;
use std::thread;
use tokio::prelude::future;
use tokio::runtime::current_thread::Runtime as CtRuntime;
use tokio::runtime::Runtime as MtRuntime;
use tokio_starvation::{client, server};

fn main() {
    let addr: SocketAddr = "127.0.0.1:15123".parse().unwrap();
    let mut server_rt = CtRuntime::new().unwrap();
    server::run(&addr, &mut server_rt).unwrap();

    let mut clients_rt = MtRuntime::new().unwrap();

    (0..16).for_each(|_| {
        let executor = clients_rt.executor();
        thread::spawn(move || {
            client::run(&addr, &executor).unwrap();
        });
    });
    server_rt.run().unwrap();
    clients_rt.block_on(future::empty::<(), ()>()).unwrap();
}
