#[cfg(test)]
mod test {
    use std::cell::{Cell, RefCell};

    // Example of Interior Mutability with Cell
    // Note: The variables "before" and "after" may not hold the same value due to interior mutability,
    // meaning it's indeterminate whether references 'a' and 'b' point to the same underlying value.
    #[allow(dead_code)]
    fn f1(a: &Cell<i32>, b: &Cell<i32>) {
        let before = a.get();
        b.set(b.get() + 1);
        let after = a.get();
        if before != after {
            println!("Will I execute?"); // Might Happen
        }
    }

    // Example of Interior Mutability with RefCell
    // Demonstrates interior mutability using RefCell, which permits mutable borrowing of its contents.
    #[allow(dead_code)]
    fn f2(v: &RefCell<Vec<i32>>) {
        let mut m = v.borrow_mut();
        m.push(1);
    }
}
