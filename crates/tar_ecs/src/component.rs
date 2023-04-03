use std::{alloc::Layout, mem::needs_drop};

use tar_ecs_macros::{identifier, Component};

use crate::callback::{CallbackFunc, CallbackId, Callbacks};

pub type ComponentName = &'static str;

/// A [`Component`] is nothing more but data, which can be stored in a given
/// [`World`](crate::world::World) on an [`Entity`](crate::entity::Entity). [`Component`] can
/// be derived via `#[derive(Component)]`.
///
/// Read further: [`Bundle`]
///
/// SAFETY:
/// - Manual implementation is discouraged
pub unsafe trait Component: Sized + Send + Sync + 'static {
    const NAME: ComponentName;

    fn get_info() -> ComponentInfo;
}

#[derive(Component)]
pub struct Empty;

identifier!(ComponentId, u32);

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
}
