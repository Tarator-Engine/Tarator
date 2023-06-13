use std::{alloc::Layout, mem::{self, needs_drop}, any};

use tar_ecs_macros::identifier;

use crate::callback::{ Callback, CallbackFunc, CallbackId, Callbacks };

pub use crate::macros::Component;
/// A [`Component`] is nothing more but data, which can be stored in a given
/// [`World`](crate::world::World) on an [`Entity`](crate::entity::Entity). [`Component`] can
/// be derived via `#[derive(Component)]`.
///
/// Read further: [`Bundle`](crate::bundle::Bundle)
///
/// SAFETY:
/// - Manual implementation is discouraged
pub unsafe trait Component: Sized + Send + Sync + 'static {
    fn uid() -> UComponentId {
        unsafe { mem::transmute(any::TypeId::of::<Self>()) }
    }
}

unsafe impl Component for () {}


identifier!(ComponentId, u32);
identifier!(UComponentId, u64);


#[derive(Debug)]
pub struct ComponentInfo {
    drop: Option<unsafe fn(*mut u8)>,
    layout: Layout,
    callbacks: Callbacks,
}

impl ComponentInfo {
    #[inline]
    pub const fn new(layout: Layout, drop: Option<unsafe fn(*mut u8)>) -> Self {
        Self {
            drop,
            layout,
            callbacks: Callbacks::new(),
        }
    }

    #[inline]
    pub fn new_from<T: Component>() -> Self {
        unsafe fn drop<T>(data: *mut u8) {
            data.cast::<T>().drop_in_place()
        }

        Self::new(Layout::new::<T>(), needs_drop::<T>().then_some(drop::<T>))
    }

    #[inline]
    pub const fn drop(&self) -> Option<unsafe fn(*mut u8)> {
        self.drop
    }

    #[inline]
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    #[inline]
    pub fn get_callback(&self, id: CallbackId) -> Option<CallbackFunc> {
        self.callbacks.get(id)
    }

    #[inline]
    pub unsafe fn set_callback(&mut self, id: CallbackId, func: CallbackFunc) {
        self.callbacks.add(id, func)
    }

    #[inline]
    pub fn set_callback_from<T: Callback<U>, U: Component>(&mut self, id: CallbackId) {
        self.callbacks.add_from::<T, U>(id)
    }
}
