use atomic_wait::{wait, wake_one};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

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

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_wait::{wait, wake_all, wake_one};
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::Relaxed;
    use std::thread;
    use std::time::Duration;

    struct Condvar {
        counter: AtomicU32,
        num_waiters: AtomicU32,
    }

    impl Condvar {
        pub const fn new() -> Self {
            Self {
                counter: AtomicU32::new(0),
                num_waiters: AtomicU32::new(0),
            }
        }

        pub fn notify_one(&self) {
            if self.num_waiters.load(Relaxed) > 0 {
                self.counter.fetch_add(1, Relaxed);
                wake_one(&self.counter);
            }
        }

        pub fn notify_all(&self) {
            if self.num_waiters.load(Relaxed) > 0 {
                self.counter.fetch_add(1, Relaxed);
                wake_all(&self.counter);
            }
        }

        pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
            self.num_waiters.fetch_add(1, Relaxed);

            let counter_value = self.counter.load(Relaxed);

            let mutex = guard.mutex;
            drop(guard);

            wait(&self.counter, counter_value);

            self.num_waiters.fetch_sub(1, Relaxed);

            mutex.lock()
        }
    }

    #[test]
    fn test_condvar() {
        let mutex = Mutex::new(0);
        let condvar = Condvar::new();

        let mut wakeups = 0;

        thread::scope(|s| {
            s.spawn(|| {
                thread::sleep(Duration::from_secs(1));
                *mutex.lock() = 123;
                condvar.notify_one();
            });

            let mut m = mutex.lock();
            while *m < 100 {
                m = condvar.wait(m);
                wakeups += 1;
            }

            assert_eq!(*m, 123);
        });
        assert!(wakeups < 10);
    }
}
