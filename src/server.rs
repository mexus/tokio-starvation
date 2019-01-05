use crate::{read_packet, yielding::Yielding};
use byteorder::{ByteOrder, BE};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::{
    future::{loop_fn, Loop},
    *,
};
use tokio::runtime::current_thread::Runtime;

fn calculate(mut data: Vec<u8>) -> Vec<u8> {
    let sum: u64 = data.iter().map(|v| u64::from(*v)).sum();
    data.clear();
    data.resize(8, 0);
    BE::write_u64(&mut data, sum);
    data
}

fn process_client_yield(
    s: TcpStream,
    id: usize,
) -> Box<Future<Item = (), Error = io::Error> + Send> {
    let buf = Vec::new();
    Box::new(loop_fn((s, buf), move |(stream, buf)| {
        Yielding::new(read_packet(stream, buf))
            .map(move |(stream, buf)| {
                let buf = calculate(buf);
                println!("[server] Got buffer from client #{}, crc = {:x?}", id, buf);
                (stream, buf)
            })
            .map(Loop::Continue::<(), _>)
    }))
}

fn process_client_no_yield(
    s: TcpStream,
    id: usize,
) -> Box<Future<Item = (), Error = io::Error> + Send> {
    let buf = Vec::new();
    Box::new(loop_fn((s, buf), move |(stream, buf)| {
        read_packet(stream, buf)
            .map(move |(stream, buf)| {
                let buf = calculate(buf);
                println!("[server] Got buffer from client #{}, crc = {:x?}", id, buf);
                (stream, buf)
            })
            .map(Loop::Continue::<(), _>)
    }))
}

pub fn run(addr: &SocketAddr, rt: &mut Runtime, should_yield: bool) -> io::Result<SocketAddr> {
    let processor: Box<dyn Fn(_, _) -> _> = if should_yield {
        Box::new(process_client_yield)
    } else {
        Box::new(process_client_no_yield)
    };
    let listener = TcpListener::bind(&addr)?;
    let addr = listener.local_addr()?;
    let id = AtomicUsize::new(0);
    let handle = rt.handle();
    let f = listener
        .incoming()
        .for_each(move |stream| {
            let id = id.fetch_add(1, Ordering::Relaxed);
            handle
                .spawn(
                    processor(stream, id)
                        .map_err(move |e| eprintln!("[server] error on client #{}: {:?}", id, e)),
                )
                .unwrap();
            Ok(())
        })
        .map_err(|e| eprintln!("[server] error: {:?}", e));
    rt.spawn(f);
    Ok(addr)
}
