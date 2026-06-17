use crate::task::IoToken;
use mio::{Events, Interest, Poll, Token};
use std::io;
use std::time::Duration;

pub struct MioPoller {
    poll: Poll,
    events: Events,
}

impl MioPoller {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            poll: Poll::new()?,
            events: Events::with_capacity(1024),
        })
    }

    pub fn registry(&self) -> &mio::Registry {
        self.poll.registry()
    }

    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<Vec<ReadyEvent>> {
        self.poll.poll(&mut self.events, timeout)?;

        Ok(self
            .events
            .iter()
            .map(|event| ReadyEvent {
                token: IoToken(event.token().0),
                readable: event.is_readable(),
                writable: event.is_writable(),
            })
            .collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadyEvent {
    pub token: IoToken,
    pub readable: bool,
    pub writable: bool,
}

pub fn mio_token(token: IoToken) -> Token {
    Token(token.0)
}

pub fn mio_interest(readable: bool, writable: bool) -> Interest {
    match (readable, writable) {
        (true, true) => Interest::READABLE | Interest::WRITABLE,
        (true, false) => Interest::READABLE,
        (false, true) => Interest::WRITABLE,
        (false, false) => Interest::READABLE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poller_can_be_created() {
        let _poller = MioPoller::new().expect("mio poller");
    }

    #[test]
    fn io_token_maps_to_mio_token() {
        assert_eq!(mio_token(IoToken(11)), Token(11));
    }
}
