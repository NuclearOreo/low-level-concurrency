#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;

    struct SpinLock<T> {
        locked: AtomicBool,
        value: UnsafeCell<T>,
    }

    impl<T> SpinLock<T> {
        pub fn new(value: T) -> Self {
            Self {
                locked: AtomicBool::new(false),
                value: UnsafeCell::new(value),
            }
        }

        pub fn lock(&self) -> &mut T {
            while self.locked.swap(true, Acquire) {
                std::hint::spin_loop();
            }
            unsafe { &mut *self.value.get() }
        }

        pub unsafe fn unlock(&self) {
            self.locked.swap(false, Release);
        }
    }

    unsafe impl<T> Sync for SpinLock<T> where T: Send {}

    #[test]
    fn f() {
        let x: SpinLock<Vec<i32>> = SpinLock::new(Vec::new());

        thread::scope(|s| {
            s.spawn(|| {
                let v = x.lock();
                v.push(1);
                unsafe { x.unlock() }
            });
            s.spawn(|| {
                let v = x.lock();
                v.push(2);
                v.push(2);
                unsafe { x.unlock() }
            });
        });

        let g = x.lock();
        assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
    }
}
