use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU16, AtomicU8, AtomicUsize, Ordering},
};

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

pub struct AtomicOption<T> {
    opt: AtomicPtr<T>,
}

impl<T> Drop for AtomicOption<T> {
    fn drop(&mut self) {
        let ptr = self.opt.load(Ordering::Relaxed);
        if !ptr.is_null() {
            drop(unsafe { Box::from_raw(ptr) })
        }
    }
}

impl<T> AtomicOption<T> {
    pub fn new(value: Option<T>) -> Self {
        Self {
            opt: AtomicPtr::new(if let Some(v) = value {
                Box::into_raw(Box::new(v))
            } else {
                null_mut()
            }),
        }
    }

    pub fn get(&self) -> Option<&T> {
        let ptr = self.opt.load(Ordering::Relaxed);
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { (&*ptr) as &T })
        }
    }

    pub fn set(&self, value: Option<T>) {
        self.opt.store(
            value
                .map(|v| Box::into_raw(Box::new(v)))
                .unwrap_or_else(null_mut),
            Ordering::Relaxed,
        )
    }
}

impl_easy_atomic!(AtomicBool, bool);
impl_easy_atomic!(AtomicUsize, usize);
impl_easy_atomic!(AtomicU16, u16);
impl_easy_atomic!(AtomicU8, u8);
