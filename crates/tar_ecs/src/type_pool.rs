use std::{
    mem::size_of,
    alloc::{ Layout, System, GlobalAlloc },
    slice::{ from_raw_parts, from_raw_parts_mut }, ptr::write_bytes,
};

pub struct TypePool {
    len: usize,
    layout: Layout,
    data: *mut u8
}

impl TypePool {
    pub unsafe fn new<T>(len: usize) -> Self {
        let layout = Layout::array::<T>(len).unwrap();
        let data = System.alloc(layout);
        data.write_bytes(0, len * size_of::<T>());
        Self { len, layout, data }
    }

    pub unsafe fn get<T>(&self, index: usize) -> &T {
        from_raw_parts(self.data.cast::<T>(), self.len).get_unchecked(index)
    }

    pub unsafe fn get_mut<T>(&self, index: usize) -> &mut T {
        from_raw_parts_mut(self.data.cast::<T>(), self.len).get_unchecked_mut(index)
    }

    pub unsafe fn clear<T>(&self, index: usize) {
        write_bytes(from_raw_parts_mut(self.data.cast::<T>(), self.len).get_unchecked_mut(index), 0, size_of::<T>())
    }

    pub unsafe fn as_slice<T>(&self) -> &[T] {
        from_raw_parts(self.data.cast::<T>(), self.len)
    }

    pub unsafe fn as_slice_mut<T>(&self) -> &mut [T] {
        from_raw_parts_mut(self.data.cast::<T>(), self.len)
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for TypePool {
    fn drop(&mut self) {
        unsafe { System.dealloc(self.data, self.layout) }
    }
}

