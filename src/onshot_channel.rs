use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;
const READING: u8 = 3;

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    state: AtomicU8,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicU8::new(EMPTY),
        }
    }
    /// Panics when more than one message is send
    pub fn send(&self, message: T) {
        if self
            .state
            .compare_exchange(EMPTY, WRITING, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            panic!("can't send more than one message!");
        }

        // Safety: only one message can be send at a time
        // and no other reference to `self.message`
        // meaning the function has exclusive (mutable)
        // reference to `self.message`.
        unsafe {
            (*self.message.get()).write(message);
        }

        self.state.store(READY, Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.state.load(Ordering::Relaxed) == READY
    }

    /// Panics if no message is available yet,
    /// or if the message was already consumed.
    ///
    /// Tip: Use `is_ready()` to check first.
    pub fn receive(&self) -> T {
        if self
            .state
            .compare_exchange(READY, READING, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            panic!("no message available!");
        }

        // Safety: We've just checked the ready flag.
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == READY {
            // Safety: `self.ready` is `READY`, so `self.message`
            // is initialized.
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;
    #[test]
    fn channel_test() {
        let channel = Channel::new();

        let t = thread::current();
        thread::scope(|s| {
            s.spawn(|| {
                channel.send("hello, world");
                t.unpark();
            });
            if !channel.is_ready() {
                thread::park();
            }

            assert_eq!(channel.receive(), "hello, world");
        })
    }
}
