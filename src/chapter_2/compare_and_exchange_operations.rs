#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicU32, AtomicU64};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Example of ID Without Overflow, using compare exchange model
    // Note: This section demonstrates the use of `compare_exchange_weak` for ID allocation without allowing overflow.
    // The `NEXT_ID` atomic variable is used to ensure that each thread can attempt to fetch and increment the ID atomically.
    // The `compare_exchange_weak` method is used here to handle concurrent updates more efficiently than `compare_exchange`.
    // This method might fail spuriously and does not guarantee that the first successful update will be made by the thread that sees the original value.
    // If the current ID is already taken (i.e., another thread has already updated the ID), the operation will fail,
    // and the thread will receive the updated current ID, retrying until it succeeds without exceeding the maximum limit of 1000 IDs.
    // The use of `Relaxed` ordering is sufficient for this example as the operation does not require synchronization with other memory operations.
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    #[test]
    fn id_allocation_without_overflow() {
        fn id_allocation() -> u32 {
            let mut id = NEXT_ID.load(Relaxed);
            loop {
                assert!(id < 1000, "Too many IDs!");
                match NEXT_ID.compare_exchange_weak(id, id + 1, Relaxed, Relaxed) {
                    Ok(_) => return id,
                    Err(v) => id = v,
                }
            }
        }

        thread::scope(|s| {
            for _ in 0..1000 {
                s.spawn(|| id_allocation());
            }
        });

        let id = NEXT_ID.load(Relaxed);
        println!("{:?}", id);
    }

    // Example of Lazy Initialization with Compare and Exchange model
    // Note: This section is intended to provide a detailed explanation of the lazy initialization example using the compare and exchange model.
    // The `AtomicU64` `KEY` is used to store a lazily initialized unique key.
    // The `get_key` function checks if the key is initialized (i.e., not zero). If not, it generates a new random key using `generate_random_key`.
    // The `compare_exchange` method is used to set the new key if the current key is still zero.
    // This method ensures that even if multiple threads attempt to initialize the key simultaneously, only one will succeed in writing the new key.
    // The others will automatically receive the newly set key, ensuring that all threads see the same key value.
    // The use of `Relaxed` ordering is sufficient as the operation does not require synchronization with other memory operations.
    // This example demonstrates a simple and efficient way to implement lazy initialization in a multi-threaded environment.
    static KEY: AtomicU64 = AtomicU64::new(0);
    #[test]
    fn get_key_with_compare_exchange() {
        pub fn generate_random_key() -> u64 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos() as u64;

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Combine timestamp and nanos with a bit of manipulation to increase randomness
            (timestamp << 32) ^ nanos
        }

        fn get_key() -> u64 {
            let key = KEY.load(Relaxed);
            if key == 0 {
                let new_key = generate_random_key();
                match KEY.compare_exchange(0, new_key, Relaxed, Relaxed) {
                    Ok(_) => new_key,
                    Err(k) => k,
                }
            } else {
                key
            }
        }

        let val1 = thread::spawn(|| get_key()).join().unwrap();
        let val2 = thread::spawn(|| get_key()).join().unwrap();

        println!("Key: {val1}");
        assert_eq!(val1, val2);
    }
}
