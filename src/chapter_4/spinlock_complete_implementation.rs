#[cfg(test)]
mod test {
    use std::cell::UnsafeCell;
    use std::ops::{Deref, DerefMut};
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

        pub fn lock(&self) -> Guard<T> {
            while self.locked.swap(true, Acquire) {
                std::hint::spin_loop();
            }
            Guard { lock: self }
        }
    }

    unsafe impl<T> Sync for SpinLock<T> where T: Send {}

    struct Guard<'a, T> {
        lock: &'a SpinLock<T>,
    }

    impl<T> Deref for Guard<'_, T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe { &*self.lock.value.get() }
        }
    }

    impl<T> DerefMut for Guard<'_, T> {
        fn deref_mut(&mut self) -> &mut T {
            unsafe { &mut *self.lock.value.get() }
        }
    }

    impl<T> Drop for Guard<'_, T> {
        fn drop(&mut self) {
            self.lock.locked.store(false, Release);
        }
    }

    #[test]
    fn f() {
        let x: SpinLock<Vec<i32>> = SpinLock::new(Vec::new());

        thread::scope(|s| {
            s.spawn(|| {
                let mut v = x.lock();
                v.push(1);
            });
            s.spawn(|| {
                let mut v = x.lock();
                v.push(2);
                v.push(2);
            });
        });

        let g = x.lock();
        assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
    }
}
