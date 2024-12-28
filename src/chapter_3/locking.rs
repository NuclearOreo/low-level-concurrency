#[cfg(test)]
mod test {
    // Example of Locking
    // This example demonstrates a basic low-level locking mechanism using an AtomicBool.
    // The `LOCK` variable acts as a binary semaphore to control access to the shared resource `DATA`.
    // The `compare_exchange` method is used to attempt to set the `LOCK` from `false` to `true`.
    // If `compare_exchange` is successful, it means the lock was acquired and the thread can safely modify `DATA`.
    // After modifying `DATA`, the lock is released by setting `LOCK` to `false` using `store` with `Release` ordering.
    // This ensures that subsequent operations in other threads see `DATA` modifications only after the lock is released.
    // The `Acquire` ordering in `compare_exchange` ensures that modifications to `DATA` are not reordered before the lock is acquired.
    // The `Relaxed` ordering for the failure case in `compare_exchange` means that on failure, no specific memory ordering is enforced.
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    static mut DATA: u32 = 0;
    static LOCK: AtomicBool = AtomicBool::new(false);

    fn f() {
        if LOCK.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
            unsafe {
                DATA = DATA + 1;
            }
            LOCK.store(false, Release);
        }
    }

    #[test]
    fn locking() {
        thread::scope(|s| {
            for _ in 0..100 {
                s.spawn(f);
            }
        });

        println!("Data:{}, could be 100 or not", unsafe { DATA })
    }
}
