#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;

    struct SpinLock<T> {
        lock: AtomicBool,
        value: UnsafeCell<T>,
    }

    impl SpinLock<T> {
        pub fn new(value: T) -> Self {
            Self {
                lock: AtomicBool::new(false),
                value: UnsafeCell::new(value),
            }
        }

        pub fn lock(&self) -> &mut T {
            while self.lock.swap(true, Acquire) {
                std::hint::spin_loop();
            }
            unsafe { &mut *self.value.get() }
        }

        pub unsafe fn unlock(&self) {
            self.lock.swap(false, Release);
        }
    }
}
