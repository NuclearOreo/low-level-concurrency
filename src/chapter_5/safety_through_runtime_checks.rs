#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        in_use: AtomicBool,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                in_use: AtomicBool::new(false),
                ready: AtomicBool::new(false),
            }
        }

        pub fn send(&self, message: T) {
            if self.in_use.swap(true, Relaxed) {
                panic!("Can't send more than one more message!");
            }
            unsafe {
                (*self.message.get()).write(message);
            }
            self.ready.store(true, Release);
        }

        pub fn is_ready(&self) -> bool {
            self.ready.load(Relaxed)
        }

        pub fn receive(&self) -> T {
            if !self.ready.load(Acquire) {
                panic!("No Message available");
            }
            unsafe { (*self.message.get()).assume_init_read() }
        }
    }

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe {
                    self.message.get_mut().assume_init_drop();
                }
            }
        }
    }

    #[test]
    fn test_channel() {
        let channel = Channel::new();

        thread::scope(|s| {
            s.spawn(|| {
                while !channel.is_ready() {}
                let val = channel.receive();
                assert_eq!(val, "Hello World");
            });
            s.spawn(|| {
                channel.send("Hello World");
            });
        });
    }
}
