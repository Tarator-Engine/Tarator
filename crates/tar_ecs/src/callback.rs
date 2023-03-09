use crate::{
    component::Component,
    store::sparse::{
        SparseSetIndex,
        MutSparseSet
    }
};

pub trait Callback<T: Component>: Sized + 'static {
    fn callback(&mut self, component: &mut T);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CallbackId(u32);

impl CallbackId {
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl SparseSetIndex for CallbackId {
    #[inline]
    fn from_usize(value: usize) -> Self {
        Self::new(value)
    }

    #[inline]
    fn as_usize(&self) -> usize {
        self.index()
    }
}


pub type CallbackFunc = unsafe fn(*mut u8, *mut u8);

pub struct ComponentCallbacks {
    callbacks: MutSparseSet<CallbackId, CallbackFunc>
}

impl ComponentCallbacks {
    #[inline]
    pub fn new() -> Self {
        Self {
            callbacks: MutSparseSet::new()
        }
    }

    unsafe fn inner_callback<T: Callback<U>, U: Component>(callback: *mut u8, component: *mut u8) {
        T::callback(&mut *callback.cast::<T>(), &mut *component.cast::<U>())
    }

    #[inline]
    pub fn add<T: Callback<U>, U: Component>(&mut self, id: CallbackId) {
        self.callbacks.insert(id, Self::inner_callback::<T, U>);
    }

    #[inline]
    pub fn get(&self, id: CallbackId) -> Option<CallbackFunc> {
        self.callbacks.get(id).map(|func| *func)
    }
}
