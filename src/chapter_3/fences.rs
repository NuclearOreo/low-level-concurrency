#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::atomic::{fence, AtomicBool};
    use std::thread;
    use std::time::Duration;

    static mut DATA: [u64; 10] = [0; 10];

    const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
    static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

    fn some_calculation() -> u64 {
        thread::sleep(Duration::from_millis(500));
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let seed = now.as_secs() ^ now.subsec_nanos() as u64;
        let mut x = seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }

    #[test]
    fn fences() {
        for i in 0..10 {
            thread::spawn(move || {
                let data = some_calculation();
                unsafe { DATA[i] = data };
                READY[i].store(true, Release);
            });
        }

        thread::sleep(Duration::from_millis(500));
        let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Relaxed));
        if ready.contains(&true) {
            fence(Acquire);
            for i in 0..10 {
                if ready[i] {
                    println!("data{i}, = {}", unsafe { DATA[i] });
                }
            }
        }
    }
}
