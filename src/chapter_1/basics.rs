#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    fn f() {
        println!("Hello from thread!");
        let id = thread::current().id();
        println!("Thread ID: {id:?}");
    }

    // Basics of spawning threads and printing their thread id
    // Note: Threads are not guaranteed to run in order
    // Note: Main thread will not wait for the spawned threads to finish
    #[test]
    fn spawn_threads() {
        println!("Hello from main thread!");
        thread::spawn(f);
        thread::sleep(Duration::from_secs(1));
        thread::spawn(f);
    }

    // Using join to wait for threads to finish
    #[test]
    fn spawn_threads_with_join() {
        let handle1 = thread::spawn(f);
        let handle2 = thread::spawn(f);

        println!("Hello from main thread!");

        handle1.join().unwrap();
        handle2.join().unwrap();
    }

    // Using closures to spawn threads
    // Note: Using move to move the variable to the closure, passing ownership to the thread
    // Note: Using move will move the variable to the thread, so the variable is no longer available in the main thread
    #[test]
    fn spawn_threads_with_closure() {
        let numbers = [1, 2, 3, 4, 5];
        let handle = thread::spawn(move || {
            for number in &numbers {
                println!("{number}")
            }
        });
        handle.join().unwrap();
    }

    // Using threads to get the average of a list of numbers
    // Note: the return value of the thread will be moved to the main thread
    #[test]
    fn get_average_using_threads() {
        let numbers = Vec::from_iter(0..=1000);
        let average = thread::spawn(move || {
            let len = numbers.len();
            let sum = numbers.iter().sum::<usize>();
            sum / len
        })
        .join()
        .unwrap();

        println!("Average: {average}");
    }
}
