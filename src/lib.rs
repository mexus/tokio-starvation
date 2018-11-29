extern crate byteorder;
extern crate rand;
extern crate rand_chacha;
extern crate tokio;

use byteorder::{ByteOrder, BE};
use std::io;
use tokio::io::{read_exact, write_all};
use tokio::prelude::*;

pub mod client;
pub mod server;
pub mod yielding;

pub(crate) fn read_packet<S: AsyncRead>(
    stream: S,
    mut buf: Vec<u8>,
) -> impl Future<Item = (S, Vec<u8>), Error = io::Error> {
    read_exact(stream, [0; 2]).and_then(move |(stream, len_bytes)| {
        let len = BE::read_u16(&len_bytes);
        buf.clear();
        buf.resize(len as usize, 0);
        read_exact(stream, buf)
    })
}

pub(crate) fn write_packet<S: AsyncWrite>(
    stream: S,
    buf: Vec<u8>,
) -> impl Future<Item = (S, Vec<u8>), Error = io::Error> {
    let mut len_bytes = [0; 2];
    assert!(buf.len() <= u16::max_value() as usize);
    BE::write_u16(&mut len_bytes, buf.len() as u16);
    write_all(stream, len_bytes).and_then(move |(stream, _)| write_all(stream, buf))
}
