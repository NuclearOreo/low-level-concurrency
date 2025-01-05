#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread::{self, Thread};

    struct Sender<'a, T> {
        channel: &'a Channel<T>,
        receiving_thread: Thread,
    }

    impl<T> Sender<'_, T> {
        pub fn send(self, message: T) {
            unsafe {
                (*self.channel.message.get()).write(message);
            }
            self.channel.ready.store(true, Release);
            self.receiving_thread.unpark();
        }
    }

    struct Receiver<'a, T> {
        channel: &'a Channel<T>,
        _no_send: PhantomData<*const ()>,
    }

    impl<T> Receiver<'_, T> {
        pub fn receive(self) -> T {
            while !self.channel.ready.swap(false, Acquire) {
                thread::park();
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

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                ready: AtomicBool::new(false),
            }
        }

        pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
            *self = Self::new();
            (
                Sender {
                    channel: self,
                    receiving_thread: thread::current(),
                },
                Receiver {
                    channel: self,
                    _no_send: PhantomData,
                },
            )
        }
    }

    #[test]
    fn test_channel() {
        let mut channel = Channel::new();
        let (sender, receiver) = channel.split();

        thread::scope(|s| {
            s.spawn(|| {
                sender.send("Wow");
            });
            assert_eq!(receiver.receive(), "Wow");
        });
    }
}
