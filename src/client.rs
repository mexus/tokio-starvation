use super::write_packet;
use future::{loop_fn, Executor, Loop};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::prelude::*;

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
        .map(|(stream, buf)| {
            println!("[client] send {} bytes", buf.len());
            (stream, rnd, buf)
        })
        .map(Loop::Continue::<(), _>)
}

pub fn run<E: Executor<Box<Future<Item = (), Error = ()> + Send + Sync>>>(
    addr: &SocketAddr,
    rt: &E,
) -> io::Result<()> {
    let rnd = ChaChaRng::from_rng(rand::thread_rng()).unwrap();
    let buf = Vec::new();

    let f = TcpStream::connect(&addr)
        .and_then(|stream| loop_fn((stream, rnd, buf), process))
        .map_err(|e| eprintln!("{:?}", e));
    rt.execute(Box::new(f)).unwrap();
    Ok(())
}
