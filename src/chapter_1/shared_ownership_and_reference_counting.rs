#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    // Shared ownership with statics
    // Statics are shared across threads and the entire program
    // Static exist for the entire lifetime of the program
    // Statics exist before the main function is called
    static X: [i32; 3] = [1, 2, 3];
    #[test]
    fn shared_ownership_with_statics() {
        thread::spawn(|| dbg!(&X)).join().unwrap();
        thread::spawn(|| dbg!(&X)).join().unwrap();
    }

    // Shared ownership with leaky references
    // Leaky references are shared across threads and the entire program
    // The "move" keyword is used to transfer ownership of the "reference" to the thread
    // The "Box::leak" function is used to create a leaky reference
    #[test]
    fn shared_ownership_with_leaky_references() {
        let x: &'static [i32; 3] = Box::leak(Box::new([1, 2, 3]));
        thread::spawn(move || dbg!(x)).join().unwrap();
        thread::spawn(move || dbg!(x)).join().unwrap();
    }

    // Shared ownership with Arc
    // Arc is a reference counting pointer
    // Arc is used in multi-threaded context
    #[test]
    fn shared_ownership_with_arc() {
        let x = Arc::new([1, 2, 3]);
        let x1 = x.clone();
        let x2 = x.clone();

        assert_eq!(x1.as_ptr(), x2.as_ptr());

        thread::spawn(move || dbg!(x1)).join().unwrap();
        thread::spawn(move || dbg!(x2)).join().unwrap();
    }
}
