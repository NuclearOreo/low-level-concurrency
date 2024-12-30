#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::sync::{Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    // Example of using mutex with threads
    // This test demonstrates the use of a Mutex to synchronize access to a shared resource across multiple threads.
    // A Mutex (mutual exclusion) is used here to prevent data races by ensuring that only one thread can access the shared data (`value`) at a time.
    // Each thread will attempt to lock the Mutex, perform operations, and then release the lock.
    #[test]
    fn accumulating_with_mutex() {
        let value = Mutex::new(0);

        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = value.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                });
            }
        });

        assert_eq!(value.into_inner().unwrap(), 1000)
    }

    // Example of Thread Parking
    // This test demonstrates the use of thread parking and unparking.
    // Thread parking is a way to put a thread to sleep (park it) until it is explicitly woken up (unparked).
    // The consuming thread will park itself if the queue is empty and will be unparked by the producing thread when a new item is pushed.
    // This ensures that the consuming thread only runs when there is data to process and sleeps otherwise, reducing CPU usage.
    #[test]
    fn parking_threads() {
        let queue = Mutex::new(VecDeque::new());

        thread::scope(|s| {
            // Consuming thread
            let t = s.spawn(|| loop {
                let item = queue.lock().unwrap().pop_front();
                if let Some(item) = item {
                    dbg!(item);
                } else {
                    thread::park();
                }
            });

            // Producing thread
            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                t.thread().unpark();
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    // Example of thread conditionals
    // This test demonstrates the use of condition variables in conjunction with Mutexes.
    // Condition variables allow threads to wait for certain conditions to become true.
    // They are used here to synchronize the consumer thread with the producer, ensuring that the consumer waits until the queue is not empty.
    // The consumer thread waits on the condition variable until the producer thread notifies it after adding an item to the queue.
    // This pattern helps in avoiding busy-waiting and makes the inter-thread communication more efficient.
    #[test]
    fn thread_conditional() {
        let queue = Mutex::new(VecDeque::new());
        let not_empty = Condvar::new();

        thread::scope(|s| {
            // Consuming thread
            s.spawn(|| loop {
                let mut q = queue.lock().unwrap();
                let item = loop {
                    if let Some(item) = q.pop_front() {
                        break item;
                    } else {
                        q = not_empty.wait(q).unwrap();
                    }
                };
                drop(q);
                dbg!(item);
            });

            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                not_empty.notify_one();
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}
