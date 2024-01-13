use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;

const LOCKED: bool = true;
const UNLOCKED: bool = false;
pub struct Mutex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }
    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
        .locked.compare_exchange_weak(
            UNLOCKED, 
            LOCKED, 
            Ordering::Relaxed, 
            Ordering::Relaxed)
        .is_err()
        {
            // MESI Protocol: Stay in S when locked
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::thread::yield_now();
            }
            std::thread::yield_now();
        }
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);

        ret
    }
}

use std::thread::spawn;
fn main() {
    let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));
    let handles: Vec<_> = (0..10)
        .map(|_| {
            spawn(move || {
                for _ in 0..100 {
                    l.with_lock(|v| {
                        *v += 1;
                    })
                }
            })
        }).collect();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(l.with_lock(|v| *v), 10 * 100);
}

#[test]
fn too_relaxed() {
    use std::sync::atomic::AtomicUsize;
    let x: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
    let y: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
    spawn(move || {});
    spawn(move || {});
}