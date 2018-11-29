use std::mem;
use tokio::prelude::*;

enum State<F: Future> {
    ShouldPoll(F),
    DataReady(Result<F::Item, F::Error>),
    Invalid,
}

pub struct Yielding<F: Future>(State<F>);

impl<F: Future> Yielding<F> {
    pub fn new(f: F) -> Self {
        Yielding(State::ShouldPoll(f))
    }
}

impl<F: Future> Future for Yielding<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.0 = match mem::replace(&mut self.0, State::Invalid) {
                State::ShouldPoll(mut f) => match f.poll() {
                    Ok(Async::NotReady) => State::ShouldPoll(f),
                    Ok(Async::Ready(data)) => {
                        task::current().notify();
                        State::DataReady(Ok(data))
                    }
                    Err(e) => {
                        task::current().notify();
                        State::DataReady(Err(e))
                    }
                },
                State::DataReady(res) => return res.map(Async::Ready),
                State::Invalid => panic!("Invalid state"),
            };
            match self.0 {
                State::DataReady(_) | State::ShouldPoll(_) => break,
                _ => {}
            }
        }
        Ok(Async::NotReady)
    }
}
