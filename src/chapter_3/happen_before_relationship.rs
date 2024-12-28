#[cfg(test)]
mod test {
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::Relaxed;
    use std::thread;

    static X: AtomicU32 = AtomicU32::new(0);
    static Y: AtomicU32 = AtomicU32::new(0);
    static X_Y_MISS: AtomicU32 = AtomicU32::new(0);
    static X_MISS: AtomicU32 = AtomicU32::new(0);
    static Y_MISS: AtomicU32 = AtomicU32::new(0);

    fn a() {
        X.store(10, Relaxed);
        Y.store(20, Relaxed);
    }

    fn b() {
        let x = X.load(Relaxed);
        let y = Y.load(Relaxed);

        if x == 0 && y == 0 {
            X_Y_MISS.fetch_add(1, Relaxed);
        }
        if x != 0 && y == 0 {
            Y_MISS.fetch_add(1, Relaxed);
        }
        if x == 0 && y != 0 {
            X_MISS.fetch_add(1, Relaxed);
        }

        X.store(0, Relaxed);
        Y.store(0, Relaxed);
    }

    // Example Showcasing Happen Before Relationship
    // Note: This example demonstrates the lack of synchronization between threads.
    // The `Relaxed` ordering used here does not enforce any happens-before relationship.
    // This means that the updates to `X` and `Y` in function `a` and their reads in function `b`
    // are not guaranteed to be visible in a consistent order, leading to potential race conditions.
    // The output will often show "X is missed" or "Y is missed" indicating missed updates.
    #[test]
    fn happen_before_example() {
        for _ in 0..100_000 {
            thread::scope(|s| {
                s.spawn(|| a());
                s.spawn(|| b());
            });
        }

        let x_miss_count = X_MISS.load(Relaxed);
        let y_miss_count = Y_MISS.load(Relaxed);
        let x_y_miss_count = X_Y_MISS.load(Relaxed);
        println!("X Miss: {x_miss_count}, Y Miss: {y_miss_count}, {x_y_miss_count}");
    }
}
