use crate::poll::MioPoller;
use std::cell::RefCell;
use std::io;

thread_local! {
    static THREAD_EVENT_LOOP: RefCell<LazyEventLoop> = RefCell::new(LazyEventLoop::new());
}

pub struct EventLoop {
    poller: MioPoller,
}

impl EventLoop {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            poller: MioPoller::new()?,
        })
    }

    pub fn poller(&self) -> &MioPoller {
        &self.poller
    }

    pub fn poller_mut(&mut self) -> &mut MioPoller {
        &mut self.poller
    }
}

#[derive(Default)]
pub struct LazyEventLoop {
    event_loop: Option<EventLoop>,
}

impl LazyEventLoop {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_allocated(&self) -> bool {
        self.event_loop.is_some()
    }

    pub fn get_or_init(&mut self) -> io::Result<&mut EventLoop> {
        if self.event_loop.is_none() {
            self.event_loop = Some(EventLoop::new()?);
        }

        Ok(self
            .event_loop
            .as_mut()
            .expect("event loop was initialized"))
    }
}

pub fn with_thread_event_loop<T>(f: impl FnOnce(&mut EventLoop) -> io::Result<T>) -> io::Result<T> {
    THREAD_EVENT_LOOP.with(|event_loop| {
        let mut event_loop = event_loop.borrow_mut();
        f(event_loop.get_or_init()?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_loop_is_allocated_on_demand() {
        let mut event_loop = LazyEventLoop::new();

        assert!(!event_loop.is_allocated());

        event_loop.get_or_init().expect("event loop");

        assert!(event_loop.is_allocated());
    }

    #[test]
    fn thread_event_loop_is_reused_within_thread() {
        let first = with_thread_event_loop(|event_loop| Ok(event_loop as *mut EventLoop as usize))
            .expect("first event loop");
        let second = with_thread_event_loop(|event_loop| Ok(event_loop as *mut EventLoop as usize))
            .expect("second event loop");

        assert_eq!(first, second);
    }
}
