#[cfg(test)]
mod test {
    // Example of Lazy Initialization with Indirection
    // This example demonstrates a thread-safe lazy initialization using atomic pointers.
    // The `PTR` static variable is an atomic pointer that starts as a null pointer.
    // The `get_data` function checks if the pointer is null, and if so, initializes it with a new String.
    // If another thread has already initialized the data, it discards the newly created data.
    // This ensures that all threads will see the same initialized data.
    use std::sync::atomic::AtomicPtr;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;

    static PTR: AtomicPtr<String> = AtomicPtr::new(std::ptr::null_mut());

    fn get_data() -> &'static String {
        let mut p = PTR.load(Acquire);

        if p.is_null() {
            p = Box::into_raw(Box::new("Hello".to_string()));
            if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
                drop(unsafe { Box::from_raw(p) });
                p = e;
            }
        }

        unsafe { &*p }
    }
    #[test]
    fn lazy_initialization_with_indirection() {
        let mut handler = vec![];

        for _ in 0..100 {
            let h = thread::spawn(get_data);
            handler.push(h);
        }

        for h in handler {
            let val = h.join().unwrap();
            println!("{val}");
        }
    }
}
