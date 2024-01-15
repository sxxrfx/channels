use std::thread;

use onshot_channel::Channel;

mod mutex_channel;
mod onshot_channel;
mod sender_receiver_channel;

fn main() {
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
    });
}
