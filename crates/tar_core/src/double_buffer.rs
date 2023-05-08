use std::ops::{Deref, DerefMut};
#[derive(Debug)]
pub struct DoubleBuffer<T: Clone> {
    pub state: T,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn new(init: T) -> Self {
        Self { state: init }
    }
    pub fn update_read(&mut self) -> T {
        self.state.clone()
    }
}

impl<T: Clone> DerefMut for DoubleBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<T: Clone> Deref for DoubleBuffer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
