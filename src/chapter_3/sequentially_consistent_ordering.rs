#[cfg(test)]
mod test {
    // Example of Sequentially Consistent Ordering
    // This section explains the use of sequentially consistent ordering for atomic operations.
    // Sequential consistency ensures a total ordering on all operations and is the strongest memory ordering.
    // It prevents reordering of read and write operations to atomic variables, which helps in maintaining synchronization across threads.
    // In the provided example, two threads (`a` and `b`) modify shared atomic booleans (`A` and `B`) and a shared string (`S`).
    // Each thread sets its respective boolean to true and checks the boolean of the other thread.
    // If the other thread's boolean is not set, it safely appends a character to the string `S`.
    // This setup ensures that the string `S` is modified no more than once due to the sequentially consistent ordering.
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::SeqCst;
    use std::thread;

    static A: AtomicBool = AtomicBool::new(false);
    static B: AtomicBool = AtomicBool::new(false);

    static mut S: String = String::new();

    #[test]
    fn sequentially_consistent_ordering() {
        let a = thread::spawn(|| {
            A.store(true, SeqCst);
            if !B.load(SeqCst) {
                unsafe {
                    S.push('!');
                }
            }
        });

        let b = thread::spawn(|| {
            B.store(true, SeqCst);
            if !A.load(SeqCst) {
                unsafe {
                    S.push('!');
                }
            }
        });

        a.join().unwrap();
        b.join().unwrap();

        assert_eq!(unsafe { S.len() }, 1);
    }
}
