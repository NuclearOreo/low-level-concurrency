#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use std::sync::Condvar;
    use std::sync::Mutex;
    use std::thread;

    struct Channel<T> {
        queue: Mutex<VecDeque<T>>,
        item_ready: Condvar,
    }

    impl<T> Channel<T> {
        pub fn new() -> Self {
            Self {
                queue: Mutex::new(VecDeque::new()),
                item_ready: Condvar::new(),
            }
        }
        pub fn send(&self, message: T) {
            self.queue.lock().unwrap().push_back(message);
            self.item_ready.notify_one();
        }

        pub fn receive(&self) -> T {
            let mut q = self.queue.lock().unwrap();
            loop {
                if let Some(message) = q.pop_front() {
                    return message;
                }
                q = self.item_ready.wait(q).unwrap();
            }
        }
    }

    #[test]
    fn testing_channel() {
        let channel: Channel<i32> = Channel::new();

        thread::scope(|s| {
            s.spawn(|| {
                channel.send(100);
                channel.send(200);
            });
            s.spawn(|| {
                let val1 = channel.receive();
                let val2 = channel.receive();
                println!("{val1}");
                println!("{val2}");
            });
        });
    }
}
