#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::thread;

    static DATA: AtomicU32 = AtomicU32::new(0);
    static READY: AtomicBool = AtomicBool::new(false);

    // Example of using Acquire and Release Ordering
    // This example demonstrates the use of Release and Acquire orderings to ensure proper synchronization between threads.
    // The `DATA` variable is updated in one thread and then a `READY` flag is set to true using Release ordering.
    // The main thread waits until it sees `READY` as true, using Acquire ordering, to ensure that the update to `DATA` is visible.
    #[test]
    fn guarantee_not_zero() {
        thread::spawn(|| {
            DATA.store(123, Relaxed);
            READY.store(true, Release);
        });

        while !READY.load(Acquire) {}
        let data = DATA.load(Relaxed);
        assert_eq!(data, 123);
    }

    // Unsafe version of the code above
    static mut UNSAFE_DATA: u32 = 0;
    #[test]
    fn guarantee_not_zero_unsafe() {
        thread::spawn(|| {
            unsafe { UNSAFE_DATA = 123 }
            READY.store(true, Release);
        });

        while !READY.load(Acquire) {}
        assert_eq!(unsafe { UNSAFE_DATA }, 123);
    }
}
