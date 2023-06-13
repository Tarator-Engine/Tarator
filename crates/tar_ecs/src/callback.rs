use tar_ecs_macros::identifier;

use crate::{component::Component, store::sparse::MutSparseSet};

pub use crate::macros::Callback;

/// # Safety
///
/// Manual implementations discouraged, use [`Callback`] derive
pub unsafe trait InnerCallback: Sized {
    const UID: UCallbackId;
}

/// Callbacks provide a way to run functions anonymously on components without the need having the concrete type of the component.
pub trait Callback<T: Component>: InnerCallback {
    fn callback(&mut self, _: &T) {}
    fn callback_mut(&mut self, _: &mut T) {}
}

identifier!(CallbackId, u32);
identifier!(UCallbackId, u64);

#[derive(Debug, Copy, Clone)]
pub struct CallbackFunc {
    pub(crate) func: unsafe fn(*mut u8, *const u8),
    pub(crate) func_mut: unsafe fn(*mut u8, *mut u8)
}

impl CallbackFunc {
    #[inline]
    pub const fn new(
        func: unsafe fn(*mut u8, *const u8),
        func_mut: unsafe fn(*mut u8, *mut u8)
    ) -> Self {
        Self { func, func_mut }
    }
}


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
        unsafe fn callback<T: Callback<U>, U: Component>(callback: *mut u8, component: *const u8) {
            T::callback(&mut *callback.cast::<T>(), &*component.cast::<U>())
        }

        unsafe fn callback_mut<T: Callback<U>, U: Component>(callback: *mut u8, component: *mut u8) {
            T::callback_mut(&mut *callback.cast::<T>(), &mut *component.cast::<U>())
        }

        self.add(id, CallbackFunc::new(callback::<T, U>, callback_mut::<T, U>))
    }

    #[inline]
    pub fn get(&self, id: CallbackId) -> Option<CallbackFunc> {
        self.callbacks.get(id).map(|func| *func)
    }
}

