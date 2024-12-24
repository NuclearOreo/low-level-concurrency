#[cfg(test)]
mod tests {
    use std::thread;

    // Using scoped threads to spawn threads
    // Scoped threads guarantee that threads won't outlive the scope
    // Note: Scoped threads will wait for all the threads to finish before returning
    #[test]
    fn scoped_threads() {
        let numbers = [1, 2, 3, 4, 5];
        thread::scope(|s| {
            s.spawn(|| println!("Length: {:?}", numbers.len()));
            s.spawn(|| {
                for number in &numbers {
                    println!("{number}");
                }
            });
        });
    }

    // Bad example of scoped threads updating a mutable variable
    // Note: This will cause a runtime error because the threads are accessing the same variable
    // Note: This is a race condition
    // #[test]
    // fn scoped_threads_with_mut() {
    //     let mut numbers = [1, 2, 3, 4, 5];
    //     thread::scope(|s| {
    //         s.spawn(|| {
    //             numbers.push(6);
    //         });
    //         s.spawn(|| {
    //             numbers.push(7);
    //         });
    //     });
    // }
}
