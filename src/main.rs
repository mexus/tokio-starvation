use std::{env, net::SocketAddr, thread};
use tokio::prelude::future;
use tokio::runtime::current_thread::Runtime as CtRuntime;
use tokio::runtime::Runtime as MtRuntime;
use tokio_starvation::{client, server};

fn main() {
    let should_yield = env::args()
        .nth(1)
        .map(|arg| arg.to_lowercase() == "yield")
        .unwrap_or(false);
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut server_rt = CtRuntime::new().unwrap();
    let addr = server::run(&addr, &mut server_rt, should_yield).unwrap();

    let mut clients_rt = MtRuntime::new().unwrap();

    (0..3).for_each(|_| {
        let executor = clients_rt.executor();
        thread::spawn(move || {
            client::run(&addr, &executor).unwrap();
        });
    });
    server_rt.run().unwrap();
    clients_rt.block_on(future::empty::<(), ()>()).unwrap();
}
