extern crate rand;
extern crate rand_chacha;
extern crate structopt;
extern crate tokio;
extern crate tokio_starvation;

use future::{loop_fn, Loop};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use std::io;
use std::net::SocketAddr;
use structopt::StructOpt;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::runtime::current_thread;
use tokio_starvation::{read_packet, write_packet};

#[derive(StructOpt)]
struct Opts {
    server_addr: SocketAddr,
}

fn generate(rnd: &mut impl Rng, mut buf: Vec<u8>) -> Vec<u8> {
    let len: u16 = rnd.gen();
    buf.resize(len as usize, 0);
    rnd.fill(&mut buf[..]);
    buf
}

fn process<R: Rng>(
    (stream, mut rnd, buf): (TcpStream, R, Vec<u8>),
) -> impl Future<Item = Loop<(), (TcpStream, R, Vec<u8>)>, Error = io::Error> {
    let gen = generate(&mut rnd, buf);
    write_packet(stream, gen)
        .and_then(|(stream, buf)| {
            read_packet(stream, buf).map(|(stream, buf)| {
                println!("Receveid a buf: {:x?}", buf);
                (stream, rnd, buf)
            })
        })
        .map(Loop::Continue::<(), _>)
}

fn main() -> io::Result<()> {
    let opts = Opts::from_args();

    let seed = [1; 32];
    let rnd = ChaChaRng::from_seed(seed);
    let buf = Vec::new();

    let mut rt = current_thread::Runtime::new()?;

    let f = TcpStream::connect(&opts.server_addr)
        .and_then(|stream| loop_fn((stream, rnd, buf), process))
        .map_err(|e| eprintln!("{:?}", e));
    rt.spawn(f);
    rt.run().unwrap();
    Ok(())
}
