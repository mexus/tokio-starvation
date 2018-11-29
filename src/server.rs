extern crate byteorder;
extern crate structopt;
extern crate tokio;
extern crate tokio_starvation;

use byteorder::{ByteOrder, BE};
use future::{loop_fn, Loop};
use std::io;
use std::net::SocketAddr;
use structopt::StructOpt;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::current_thread;
use tokio_starvation::{read_packet, write_packet};

#[derive(StructOpt)]
struct Opts {
    listen_addr: SocketAddr,
}

fn calculate(mut data: Vec<u8>) -> Vec<u8> {
    let sum: u64 = data.iter().map(|v| u64::from(*v)).sum();
    data.clear();
    data.resize(8, 0);
    BE::write_u64(&mut data, sum);
    data
}

fn process_client(s: TcpStream) -> impl Future<Item = (), Error = io::Error> {
    let buf = Vec::new();
    loop_fn((s, buf), move |(stream, buf)| {
        read_packet(stream, buf)
            .map(|(stream, buf)| (stream, calculate(buf)))
            .and_then(|(stream, buf)| write_packet(stream, buf))
            .map(Loop::Continue::<(), _>)
    })
}

fn main() -> io::Result<()> {
    let opts = Opts::from_args();
    let mut rt = current_thread::Runtime::new()?;
    let f = TcpListener::bind(&opts.listen_addr)?
        .incoming()
        .for_each(|stream| {
            tokio::spawn(process_client(stream).map_err(|e| eprintln!("{:?}", e)));
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));
    rt.spawn(f);
    rt.run().unwrap();
    Ok(())
}
