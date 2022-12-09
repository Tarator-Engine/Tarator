use std::{
    mem::size_of,
    alloc::{ Layout, System, GlobalAlloc },
    slice::{ from_raw_parts, from_raw_parts_mut }, ptr::write_bytes,
};
use crate::MAXENTITIES;
use super::Component;



struct TypePool {
    len: usize,
    layout: Layout,
    data: *mut u8
}

impl TypePool {
    unsafe fn new<T>(len: usize) -> Self {
        let layout = Layout::array::<T>(len).unwrap();
        let data = System.alloc(layout);
        data.write_bytes(0, len * size_of::<T>());
        Self { len, layout, data }
    }

    unsafe fn get<T>(&self, index: usize) -> &T {
        from_raw_parts(self.data.cast::<T>(), self.len).get_unchecked(index)
    }

    unsafe fn get_mut<T>(&self, index: usize) -> &mut T {
        from_raw_parts_mut(self.data.cast::<T>(), self.len).get_unchecked_mut(index)
    }

    unsafe fn clear<T>(&self, index: usize) {
        write_bytes(from_raw_parts_mut(self.data.cast::<T>(), self.len).get_unchecked_mut(index), 0, size_of::<T>())
    }

    unsafe fn as_slice<T>(&self) -> &[T] {
        from_raw_parts(self.data.cast::<T>(), self.len)
    }

    unsafe fn as_slice_mut<T>(&self) -> &mut [T] {
        from_raw_parts_mut(self.data.cast::<T>(), self.len)
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl Drop for TypePool {
    fn drop(&mut self) {
        unsafe { System.dealloc(self.data, self.layout) }
    }
}



/// Safe wrapper around TypePool. Stores one type of components.
/// TODO Track cleared components and sort data
pub(crate) struct ComponentPool {
    pool: TypePool
}

impl ComponentPool {
    pub(crate) fn new<C: Component>() -> Self {
        Self {
            pool: unsafe{TypePool::new::<C>(MAXENTITIES)}
        }
    }
    #[allow(unused)]
    pub(crate) fn get<C: Component>(&self, index: usize) -> Result<&C, String> {
        if index > self.pool.len() { return Err(format!("index({}) is out of bounds!", index)); }
        Ok(unsafe{self.pool.get::<C>(index)})
    }
    pub(crate) fn get_mut<C: Component>(&self, index: usize) -> Result<&mut C, String> {
        if index > self.pool.len() { return Err(format!("index({}) is out of bounds!", index)); }
        Ok(unsafe{self.pool.get_mut::<C>(index)})
    }
    pub(crate) fn clear<C: Component>(&self, index: usize) -> Result<(), String> {
        if index > self.pool.len() { return Err(format!(" index({}) id out of bounds!", index)); }
        Ok(unsafe{self.pool.clear::<C>(index)})
    }
    #[allow(unused)]
    pub(crate) fn as_slice<C: Component>(&self) -> &[C] {
        unsafe{self.pool.as_slice::<C>()}
    }
    #[allow(unused)]
    pub(crate) fn as_slice_mut<C: Component>(&self) -> &mut [C] {
        unsafe{self.pool.as_slice_mut::<C>()}
    }
}

