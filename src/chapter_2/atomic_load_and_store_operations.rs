#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};
    use std::thread;
    use std::time::Duration;

    // Example of a Stop Flag
    // Note: AtomicBool is used here as a stop flag for a background thread.
    // The AtomicBool `STOP` is initialized to `false` and is set to `true` to signal the thread to stop.
    // The `load` method with `Relaxed` ordering is used for checking the flag in a loop.
    // The `store` method with `Relaxed` ordering is used to update the flag from the main thread.
    // `Relaxed` ordering is used for operations on `STOP` to avoid memory ordering constraints,
    // which is acceptable here because the operations are not dependent on other memory operations.
    static STOP: AtomicBool = AtomicBool::new(false);
    #[test]
    fn stop_flag() {
        // Spawn background thread
        let background_thread = thread::spawn(|| {
            let mut count = 0;
            while !STOP.load(Relaxed) {
                count += 1;
            }
            println!("I'm Done, total loops: {count}");
        });

        // Main thread sleeps for 3 seconds
        thread::sleep(Duration::from_secs(1));

        // Toggle Atomic bool to stop the job
        STOP.store(true, Relaxed);

        // Wait until background thread finishes
        background_thread.join().unwrap();
    }

    // Example of a Progress Bar
    // Note: This section is intended to provide a detailed explanation of the progress bar example.
    // The `AtomicUsize` `num_done` is used to track the number of items processed.
    // Each item processing is simulated with a sleep of 500 milliseconds.
    // The `store` method with `Relaxed` ordering updates the progress in a non-blocking manner.
    // A separate thread is spawned to process items and update `num_done`.
    // The main thread continuously checks the progress and prints it until all items are processed.
    // The use of `Relaxed` ordering is sufficient as there are no dependencies on the order of other variables.
    // This example demonstrates a simple way to report progress of a task running on a background thread.
    #[test]
    fn progress_reporting() {
        let num_done = AtomicUsize::new(0);
        let process_item = |_| {
            thread::sleep(Duration::from_millis(500));
        };

        thread::scope(|s| {
            // Background thread to process items
            s.spawn(|| {
                for i in 0..100 {
                    process_item(i);
                    num_done.store(i + 1, Relaxed);
                }
            });
        });

        // Main thread loop to check the number of items processed
        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working.. {n}/100 finished");
            thread::sleep(Duration::from_secs(1));
        }

        println!("Done!!");
    }

    // Example of Progress Reporting with parking main thread
    // Note: This example demonstrates an alternative method of progress reporting where the main thread is parked
    // and only resumes when the background thread has made progress. This can be more efficient in scenarios
    // where the main thread does not need to perform other work while waiting for the background thread.
    // The `AtomicUsize` `num_done` is used to track the number of items processed.
    // Each item processing is simulated with a sleep of 20 milliseconds.
    // The `store` method with `Relaxed` ordering updates the progress in a non-blocking manner.
    // A separate thread is spawned to process items and update `num_done`.
    // The main thread parks itself and only resumes when unparked by the background thread, reducing busy-waiting.
    // The use of `Relaxed` ordering is sufficient as there are no dependencies on the order of other variables.
    // This example demonstrates a more synchronized way to report progress of a task running on a background thread.
    #[test]
    fn progress_reporting_with_parking() {
        let num_done = AtomicUsize::new(0);
        let process_item = |_| {
            thread::sleep(Duration::from_millis(20));
        };

        let main_thread = thread::current();

        thread::scope(|s| {
            // Background thread to process items
            s.spawn(|| {
                for i in 0..100 {
                    process_item(i);
                    num_done.store(i + 1, Relaxed);
                    main_thread.unpark();
                }
            });
        });

        // Main thread loop to check the number of items processed
        loop {
            let n = num_done.load(Relaxed);
            println!("Working.. {n}/100 finished");
            if n == 100 {
                break;
            }
            thread::park_timeout(Duration::from_nanos(1));
        }

        println!("Done!!");
    }

    // Example of Lazy Initialization using AtomicU64
    // Note: This example demonstrates the use of atomic operations for lazy initialization.
    // The `AtomicU64` `X` is used to store a lazily initialized value.
    // The `load` method with `Relaxed` ordering is used to check if the value has been initialized.
    // If the value is 0, it indicates that initialization has not occurred, and the value is then calculated and stored.
    // The `store` method with `Relaxed` ordering is used to save the calculated value.
    // This pattern ensures that the value is only calculated once and is thread-safe without the need for locks.
    static X: AtomicU64 = AtomicU64::new(0);
    #[test]
    fn get_x() {
        let calculate_x = || 100;
        let mut x = X.load(Relaxed);

        if x == 0 {
            x = calculate_x();
            X.store(x, Relaxed);
        }

        let value = X.load(Relaxed);
        assert_eq!(value, 100);
    }
}
