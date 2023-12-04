use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, AtomicUsize, Ordering};

pub trait EasyAtomic {
    type Item;

    fn get(&self) -> Self::Item;
    fn set(&self, value: Self::Item);
}

macro_rules! impl_easy_atomic {
    ($a:ty, $b:ty) => {
        impl EasyAtomic for $a {
            type Item = $b;

            fn get(&self) -> Self::Item {
                self.load(Ordering::Relaxed)
            }

            fn set(&self, value: Self::Item) {
                self.store(value, Ordering::Relaxed);
            }
        }
    };
}

impl_easy_atomic!(AtomicBool, bool);
impl_easy_atomic!(AtomicUsize, usize);
impl_easy_atomic!(AtomicU16, u16);
impl_easy_atomic!(AtomicU8, u8);
