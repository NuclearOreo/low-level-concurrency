#[cfg(test)]
mod tests {
    use atomic_wait::{wait, wake_all, wake_one};
    use std::cell::UnsafeCell;
    use std::ops::{Deref, DerefMut};
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    struct RwLock<T> {
        state: AtomicU32,
        data: UnsafeCell<T>,
    }

    unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

    impl<T> RwLock<T> {
        pub const fn new(value: T) -> Self {
            Self {
                state: AtomicU32::new(0), // 0: unlocked
                data: UnsafeCell::new(value),
            }
        }

        pub fn read(&self) -> ReadGuard<T> {
            let mut s = self.state.load(Relaxed);
            loop {
                if s < u32::MAX {
                    assert!(s != u32::MAX - 1, "too many readers");
                    match self.state.compare_exchange_weak(s, s + 1, Acquire, Relaxed) {
                        Ok(_) => return ReadGuard { rwlock: self },
                        Err(e) => s = e,
                    }
                }
                if s == u32::MAX {
                    wait(&self.state, s);
                    s = self.state.load(Relaxed);
                }
            }
        }

        pub fn write(&self) -> WriteGuard<T> {
            while let Err(s) = self.state.compare_exchange(0, u32::MAX, Acquire, Relaxed) {
                // Wait for the lock to be released
                wait(&self.state, s);
            }
            WriteGuard { rwlock: self }
        }
    }

    struct ReadGuard<'a, T> {
        rwlock: &'a RwLock<T>,
    }

    impl<T> Deref for ReadGuard<'_, T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe { &*self.rwlock.data.get() }
        }
    }

    impl<T> Drop for ReadGuard<'_, T> {
        fn drop(&mut self) {
            if self.rwlock.state.fetch_sub(1, Release) == 1 {
                wake_one(&self.rwlock.state);
            }
        }
    }

    struct WriteGuard<'a, T> {
        rwlock: &'a RwLock<T>,
    }

    impl<T> Deref for WriteGuard<'_, T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe { &*self.rwlock.data.get() }
        }
    }

    impl<T> DerefMut for WriteGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut T {
            unsafe { &mut *self.rwlock.data.get() }
        }
    }

    impl<T> Drop for WriteGuard<'_, T> {
        fn drop(&mut self) {
            self.rwlock.state.store(0, Release);
            wake_all(&self.rwlock.state);
        }
    }

    #[test]
    fn test1() {
        let rwlock = RwLock::new(0);
        *rwlock.write() += 1;
        let r1 = rwlock.read();
        assert_eq!(*r1, 1);
    }

    #[test]
    fn test2() {
        let rwlock = RwLock::new(0);

        thread::scope(|s| {
            for _ in 0..100 {
                s.spawn(|| {
                    *rwlock.write() += 2;
                });
            }
        });

        let r1 = rwlock.read();
        assert_eq!(*r1, 200);
    }
}
