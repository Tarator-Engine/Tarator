use std::{
    sync::{
        Arc,
        atomic::{ AtomicBool, Ordering }
    },
    ops::{ Deref, DerefMut }
};
use crate::error::EcsError as Error;


pub struct StorePtr<T> {
    lock: Arc<AtomicBool>,
    data: *mut T
}

impl<T> StorePtr<T> {
    fn lock(&self) -> Result<StorePtrGuard<T>, Error> {
        // TODO check here for logic errors pls
        while self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) != Ok(true) {}

        Ok(StorePtrGuard {
            lock: self.lock.clone(),
            data: self.data
        })
    }
}


pub struct StorePtrGuard<T> {
    lock: Arc<AtomicBool>,
    data: *mut T
}
impl<T> Deref for StorePtrGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}
impl<T> DerefMut for StorePtrGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut*self.data } 
    }
}
impl<T> Drop for StorePtrGuard<T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}


pub(crate) struct Store {
    lock: Arc<AtomicBool>,
}

impl Store {
}

impl Drop for Store {
    fn drop(&mut self) {
    }
}

