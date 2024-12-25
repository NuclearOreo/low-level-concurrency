#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicU64, AtomicUsize};
    use std::thread;
    use std::time::{Duration, Instant};

    // Example of Progress Reporting from Multiple Threads
    // Note: This example demonstrates progress reporting from multiple threads.
    // The `AtomicUsize` `num_done` is used to track the number of items processed by multiple threads.
    // Each item processing is simulated with a sleep of 20 milliseconds.
    // The `fetch_add` method with `Relaxed` ordering is used to atomically update the progress.
    // Four threads are spawned, each processing a quarter of the total items (25 each), and update the shared atomic counter.
    // The main thread continuously checks the progress and prints it until all items are processed.
    // The use of `Relaxed` ordering is sufficient as there are no dependencies on the order of other variables.
    // This example demonstrates a more complex scenario of progress reporting where multiple threads contribute to a shared counter.
    #[test]
    fn progress_reporting_with_multiple_threads() {
        let num_done = &AtomicUsize::new(0);
        let process_item = |_| {
            thread::sleep(Duration::from_millis(20));
        };

        thread::scope(|s| {
            // Spawn four background threads to process all 100 items, 25 each.
            for t in 0..4 {
                s.spawn(move || {
                    for i in 0..25 {
                        process_item(t * 25 + i);
                        num_done.fetch_add(1, Relaxed);
                    }
                });
            }
        });

        // Main thread loop to check the number of items processed
        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working.. {n}/100 finished");
            thread::sleep(Duration::from_millis(10));
        }

        println!("Done!!");
    }

    // Example of Statistics
    // Note: This section is intended to provide a detailed explanation of the statistics example.
    // The `AtomicUsize` `num_done` is used to track the number of items processed.
    // The `AtomicU64` `total_time` and `max_time` are used to track the total processing time and the maximum time taken for a single item, respectively.
    // Each item processing is simulated with a sleep of 20 milliseconds.
    // The `fetch_add` method with `Relaxed` ordering is used to atomically update the counters for `num_done` and `total_time`.
    // The `fetch_max` method with `Relaxed` ordering is used to update `max_time` if the current item's processing time exceeds the recorded maximum.
    // Four threads are spawned, each processing a quarter of the total items (25 each), and update the shared atomic counters.
    // The main thread continuously checks the progress and prints detailed statistics including average and peak processing times until all items are processed.
    // The use of `Relaxed` ordering is sufficient as there are no dependencies on the order of other variables.
    // This example demonstrates a more complex scenario of progress reporting where multiple threads contribute to shared counters and detailed statistics are calculated.
    #[test]
    fn statistics() {
        let num_done = &AtomicUsize::new(0);
        let total_time = &AtomicU64::new(0);
        let max_time = &AtomicU64::new(0);
        let process_item = |_| {
            thread::sleep(Duration::from_millis(20));
        };

        thread::scope(|s| {
            // Spawn four background threads to process all 100 items, 25 each.
            for t in 0..4 {
                s.spawn(move || {
                    for i in 0..25 {
                        let start = Instant::now();
                        process_item(t * 25 + i);
                        let time_taken = start.elapsed().as_micros() as u64;
                        num_done.fetch_add(1, Relaxed);
                        total_time.fetch_add(time_taken, Relaxed);
                        max_time.fetch_max(time_taken, Relaxed);
                    }
                });
            }
        });

        // Main thread loop to check the number of items processed
        loop {
            let total_time = Duration::from_micros(total_time.load(Relaxed));
            let max_time = Duration::from_micros(max_time.load(Relaxed));
            let n = num_done.load(Relaxed);
            if n == 0 {
                println!("Working.. no work done yet")
            } else {
                println!(
                    "Working.. {n}/100 finished, {:?} average, {:?} peak",
                    total_time / n as u32,
                    max_time
                );
            }
            if n == 100 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        println!("Done!!");
    }
}
