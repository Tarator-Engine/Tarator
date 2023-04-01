use tar_ecs_macros::identifier;

use crate::{
    component::Component,
    store::sparse::MutSparseSet,
};

pub type CallbackName = &'static str;

pub unsafe trait InnerCallback: Sized + Send + Sync + 'static {
    const NAME: CallbackName;
}

/// Callbacks provide a way to run functions anonymously on components without the need having the concrete type of the component.
pub trait Callback<T: Component>: InnerCallback + Sized + Send + Sync + 'static {
    fn callback(&mut self, component: &mut T);
}

identifier!(CallbackId, u32);

pub type CallbackFunc = unsafe fn(*mut u8, *mut u8);

#[derive(Debug)]
pub struct Callbacks {
    callbacks: MutSparseSet<CallbackId, CallbackFunc>,
}

impl Callbacks {
    #[inline]
    pub const fn new() -> Self {
        Self {
            callbacks: MutSparseSet::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, id: CallbackId, func: CallbackFunc) {
        self.callbacks.insert(id, func)
    }

    #[inline]
    pub fn add_from<T: Callback<U>, U: Component>(&mut self, id: CallbackId) {
        unsafe fn callback<T: Callback<U>, U: Component>(callback: *mut u8, component: *mut u8) {
            T::callback(&mut *callback.cast::<T>(), &mut *component.cast::<U>())
        }

        self.add(id, callback::<T, U>)
    }

    #[inline]
    pub fn get(&self, id: CallbackId) -> Option<CallbackFunc> {
        self.callbacks.get(id).map(|func| *func)
    }
}
