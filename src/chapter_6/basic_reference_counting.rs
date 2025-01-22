#[cfg(test)]
mod test {
    use std::ops::Deref;
    use std::ptr::NonNull;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::atomic::{fence, AtomicUsize};

    struct ArcData<T> {
        ref_count: AtomicUsize,
        data: T,
    }

    struct Arc<T> {
        ptr: NonNull<ArcData<T>>,
    }

    unsafe impl<T: Send + Sync> Send for Arc<T> {}
    unsafe impl<T: Send + Sync> Sync for Arc<T> {}

    impl<T> Arc<T> {
        pub fn new(data: T) -> Arc<T> {
            Arc {
                ptr: NonNull::from(Box::leak(Box::new(ArcData {
                    ref_count: AtomicUsize::new(1),
                    data,
                }))),
            }
        }

        fn data(&self) -> &ArcData<T> {
            unsafe { self.ptr.as_ref() }
        }

        #[allow(dead_code)]
        pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
            if arc.data().ref_count.load(Relaxed) == 1 {
                fence(Acquire);
                unsafe { Some(&mut arc.ptr.as_mut().data) }
            } else {
                None
            }
        }
    }

    impl<T> Deref for Arc<T> {
        type Target = T;

        fn deref(&self) -> &T {
            &self.data().data
        }
    }

    impl<T> Clone for Arc<T> {
        fn clone(&self) -> Self {
            if self.data().ref_count.load(Relaxed) > usize::MAX / 2 {
                std::process::abort();
            }

            self.data().ref_count.fetch_add(1, Relaxed);
            Arc { ptr: self.ptr }
        }
    }

    impl<T> Drop for Arc<T> {
        fn drop(&mut self) {
            if self.data().ref_count.fetch_sub(1, Release) == 1 {
                fence(Acquire);
                unsafe {
                    drop(Box::from_raw(self.ptr.as_ptr()));
                }
            }
        }
    }

    #[test]
    fn test() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

        struct DetectDrop;

        impl Drop for DetectDrop {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, Relaxed);
            }
        }

        // Create two Arcs sharing an object containing a string
        // and a DetectDrop, to detect when it's dropped.
        let x = Arc::new(("Hello", DetectDrop));
        let y = x.clone();

        // Send x to another thread, and use it there.
        let t = std::thread::spawn(move || {
            assert_eq!(x.0, "Hello");
        });

        // In parallel, y should still be usable here.
        assert_eq!(y.0, "Hello");

        // Wait for the thread to finish.
        t.join().unwrap();

        // One Arc, x, should be dropped by now.
        // We still have y, so the object shouldn't have been dropped yet.
        assert_eq!(NUM_DROPS.load(Relaxed), 0);

        // Drop the remaining `Arc`.
        drop(y);

        // Now the `y` is dropped too,
        // the object should've been dropped.
        assert_eq!(NUM_DROPS.load(Relaxed), 1);
    }
}
