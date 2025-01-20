#[cfg(test)]
mod tests {
    use atomic_wait::{wait, wake_one};
    use std::cell::UnsafeCell;
    use std::ops::{Deref, DerefMut};
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    struct Mutex<T> {
        /// 0: unlocked
        /// 1: locked
        /// 2: locked and waiting on other thread
        state: AtomicU32,
        value: UnsafeCell<T>,
    }

    unsafe impl<T> Sync for Mutex<T> where T: Send {}

    impl<T> Mutex<T> {
        pub const fn new(value: T) -> Self {
            Self {
                state: AtomicU32::new(0), // 0: unlocked state
                value: UnsafeCell::new(value),
            }
        }

        pub fn lock(&self) -> MutexGuard<T> {
            while self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
                lock_contended(&self.state);
            }
            MutexGuard { mutex: self }
        }
    }

    fn lock_contended(state: &AtomicU32) {
        let mut spin_count = 0;

        while state.load(Relaxed) == 1 && spin_count < 100 {
            spin_count += 1;
            std::hint::spin_loop();
        }

        if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
            return;
        }

        while state.swap(2, Acquire) != 0 {
            wait(state, 2);
        }
    }

    struct MutexGuard<'a, T> {
        mutex: &'a Mutex<T>,
    }

    impl<T> Deref for MutexGuard<'_, T> {
        type Target = T;
        fn deref(&self) -> &T {
            unsafe { &*self.mutex.value.get() }
        }
    }

    impl<T> DerefMut for MutexGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut T {
            unsafe { &mut *self.mutex.value.get() }
        }
    }

    impl<T> Drop for MutexGuard<'_, T> {
        fn drop(&mut self) {
            if self.mutex.state.swap(0, Release) == 2 {
                wake_one(&self.mutex.state);
            }
        }
    }

    #[test]
    fn test_mutex() {
        let mutex = Mutex::new(0);
        let mut guard = mutex.lock();
        *guard = 1;

        drop(guard);

        let guard = mutex.lock();
        assert_eq!(*guard, 1);
    }

    #[test]
    fn test_mutex_concurrent() {
        let mutex = Mutex::new(0);

        thread::scope(|s| {
            s.spawn(|| {
                let mut guard = mutex.lock();
                *guard += 1;
            });
            s.spawn(|| {
                let mut guard = mutex.lock();
                *guard += 2;
            });
        });

        let guard = mutex.lock();
        assert_eq!(*guard, 3);
    }
}
