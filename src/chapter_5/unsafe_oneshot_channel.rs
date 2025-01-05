#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::mem::MaybeUninit;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;

    struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                ready: AtomicBool::new(false),
            }
        }

        pub unsafe fn send(&self, message: T) {
            (*self.message.get()).write(message);
            self.ready.store(true, Release);
        }

        pub fn is_ready(&self) -> bool {
            self.ready.load(Acquire)
        }

        pub unsafe fn receive(&self) -> T {
            (*self.message.get()).assume_init_read()
        }
    }

    #[test]
    fn test_channel() {
        let channel: Channel<i32> = Channel::new();

        thread::scope(|s| {
            s.spawn(|| unsafe {
                while !channel.is_ready() {}
                let val = channel.receive();
                println!("{val}");
            });
            s.spawn(|| unsafe {
                channel.send(100);
            });
        });
    }
}
