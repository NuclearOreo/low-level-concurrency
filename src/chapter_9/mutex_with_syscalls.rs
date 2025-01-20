#[cfg(test)]
mod tests {
    use atomic_wait::{wait, wake_one};
    use std::cell::UnsafeCell;
    use std::ops::{Deref, DerefMut};
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;

    struct Mutex<T> {
        /// 0: unlocked
        /// 1: locked
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
            // Set the state to locked: 1
            while self.state.swap(1, Acquire) == 1 {
                // If it was already locked..
                // ..wait, unless the state is no longer locked
                wait(&self.state, 1);
            }
            MutexGuard { mutex: self }
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
            // Set the state to unlocked: 0
            self.mutex.state.store(0, Release);
            // Wake up one thread waiting on the mutex
            wake_one(&self.mutex.state);
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
