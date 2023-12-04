use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, AtomicUsize, Ordering};

pub trait EasyAtomic {
    type Item;

    fn get(&self) -> Self::Item;
    fn set(&self, value: Self::Item);
}

impl EasyAtomic for AtomicBool {
    type Item = bool;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed);
    }
}

impl EasyAtomic for AtomicUsize {
    type Item = usize;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed);
    }
}

impl EasyAtomic for AtomicU16 {
    type Item = u16;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed);
    }
}

impl EasyAtomic for AtomicU8 {
    type Item = u8;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed);
    }
}
