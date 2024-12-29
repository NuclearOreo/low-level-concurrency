#[cfg(test)]
mod test {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;

    struct SpinLock {
        lock: AtomicBool,
    }

    impl SpinLock {
        pub fn new() -> Self {
            Self {
                lock: AtomicBool::new(false),
            }
        }

        pub fn lock(&self) {
            while self.lock.swap(true, Acquire) {
                std::hint::spin_loop();
            }
        }

        pub fn unlock(&self) {
            self.lock.swap(false, Release);
        }
    }

    #[test]
    fn f() {
        let spinlock = SpinLock::new();

        thread::scope(|s| {
            s.spawn(|| {
                spinlock.lock();
                println!("Lock by thread 1");
                spinlock.unlock();
            });
            s.spawn(|| {
                spinlock.lock();
                println!("Lock by thread 2");
                spinlock.unlock();
            });
        });
    }
}
