use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}
pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let a = Arc::new(Channel {
        message: UnsafeCell::new(MaybeUninit::uninit()),
        ready: AtomicBool::new(false),
    });

    (Sender { channel: a.clone() }, Receiver { channel: a })
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Sender<T> {
    pub fn send(self, message: T) {
        // Safety: Only one writer and no reader
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
    }
}

impl<T> Receiver<T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Ordering::Relaxed)
    }

    pub fn receive(&self) -> T {
        if !self.channel.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!");
        }

        // Safety: No writer and only reader
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            // Safety: `self.ready` == `true` so, `self.message` is intialized
            unsafe { (*self.message.get_mut()).assume_init_drop() }
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;
    #[test]
    fn sender_receiver() {
        let (tx, rx) = channel();

        let t = thread::current();
        thread::scope(|s| {
            s.spawn(|| {
                tx.send("hello, world");
                t.unpark();
            });
            if !rx.is_ready() {
                thread::park();
            }

            assert_eq!(rx.receive(), "hello, world");
        })
    }
}
