#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::Arc;
    use std::thread;

    struct Sender<T> {
        channel: Arc<Channel<T>>,
    }

    impl<T> Sender<T> {
        pub fn send(self, message: T) {
            unsafe {
                (*self.channel.message.get()).write(message);
            }
            self.channel.ready.store(true, Release);
        }
    }

    struct Receiver<T> {
        channel: Arc<Channel<T>>,
    }

    impl<T> Receiver<T> {
        pub fn is_ready(&self) -> bool {
            self.channel.ready.load(Relaxed)
        }

        pub fn receive(self) -> T {
            if !self.channel.ready.swap(false, Acquire) {
                panic!("No message available!");
            }
            unsafe { (*self.channel.message.get()).assume_init_read() }
        }
    }

    struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe {
                    self.message.get_mut().assume_init_drop();
                }
            }
        }
    }

    fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let a = Arc::new(Channel {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        });

        (
            Sender { channel: a.clone() },
            Receiver { channel: a.clone() },
        )
    }

    #[test]
    fn test_channel() {
        let (sender, receiver) = channel();
        let t = thread::current();

        thread::scope(|s| {
            s.spawn(|| {
                sender.send("Wow");
                t.unpark();
            });
            while !receiver.is_ready() {
                thread::park();
            }
            assert_eq!(receiver.receive(), "Wow");
        });
    }
}
