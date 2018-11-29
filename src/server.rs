use byteorder::{ByteOrder, BE};
use future::{loop_fn, Loop};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;
use {read_packet, yielding::Yielding};

fn calculate(mut data: Vec<u8>) -> Vec<u8> {
    let sum: u64 = data.iter().map(|v| u64::from(*v)).sum();
    data.clear();
    data.resize(8, 0);
    BE::write_u64(&mut data, sum);
    data
}

fn process_client(s: TcpStream, id: usize) -> impl Future<Item = (), Error = io::Error> {
    let buf = Vec::new();
    loop_fn((s, buf), move |(stream, buf)| {
        Yielding::new(read_packet(stream, buf))
            .map(move |(stream, buf)| {
                let buf = calculate(buf);
                println!("[server] Got buffer from client #{}, crc = {:x?}", id, buf);
                (stream, buf)
            })
            .map(Loop::Continue::<(), _>)
    })
}

pub fn run(addr: &SocketAddr, rt: &mut Runtime) -> io::Result<()> {
    let id = AtomicUsize::new(0);
    let handle = rt.handle();
    let f = TcpListener::bind(&addr)?
        .incoming()
        .for_each(move |stream| {
            let id = id.fetch_add(1, Ordering::Relaxed);
            handle
                .spawn(
                    process_client(stream, id)
                        .map_err(move |e| eprintln!("Error on client #{}: {:?}", id, e)),
                )
                .unwrap();
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));
    rt.spawn(f);
    Ok(())
}
