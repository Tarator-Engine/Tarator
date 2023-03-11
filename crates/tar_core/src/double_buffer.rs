use std::ops::{Deref, DerefMut};

use parking_lot::Mutex;

#[derive(Debug)]
pub struct RawDoubleBuffer<T: Clone> {
    pub state: T,
}

impl<T: Clone> RawDoubleBuffer<T> {
    pub fn update_read(&mut self) -> T {
        return self.state.clone();
    }
}

impl<T: Clone> DerefMut for RawDoubleBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<T: Clone> Deref for RawDoubleBuffer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

pub struct DoubleBuffer<T: Clone> {
    raw: Mutex<RawDoubleBuffer<T>>,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn new(state: T) -> Self {
        Self {
            raw: Mutex::new(RawDoubleBuffer { state: state }),
        }
    }
}

impl<T: Clone> DerefMut for DoubleBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl<T: Clone> Deref for DoubleBuffer<T> {
    type Target = Mutex<RawDoubleBuffer<T>>;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
