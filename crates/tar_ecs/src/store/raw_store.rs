//! Big portions of this code where looked up from
//! <https://docs.rs/bevy_ecs/latest/src/bevy_ecs/storage/blob_vec.rs.html>

use std::alloc::{ Layout, handle_alloc_error };

/// Type erased vector
pub struct RawStore {
    item_layout: Layout,
    capacity: usize,
    len: usize,
    data: *mut u8,
    drop: Option<unsafe fn(*mut u8)>
}

impl std::fmt::Debug for RawStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawStore")
            .field("item_layout", &self.item_layout)
            .field("capacity", &self.capacity)
            .field("len", &self.len)
            .field("data", &self.data)
            .field("drop", &match self.drop {
                Some(_) => "Some(_)",
                None => "None"
            })
            .finish()
    }
}

impl RawStore {
    /// SAFETY:
    /// - If the items need to be dropped, `drop` must have `Some(_)`
    /// - Capacity should be > 0
    pub unsafe fn with_capacity(
        item_layout: Layout,
        drop: Option<unsafe fn(*mut u8)>,
        capacity: usize
    ) -> Self {
        if item_layout.size() == 0 {
            let data = item_layout.align() as *mut u8;
            debug_assert!(!data.is_null(), "Align must be > 0!");
            Self {
                item_layout,
                capacity: usize::MAX,
                len: 0,
                data,
                drop
            }
        } else {
            let mut store = Self {
                item_layout,
                capacity: 0,
                len: 0,
                data: std::ptr::null_mut(),
                drop
            };
            store.reserve_exact(capacity);

            store
        }
    }

    #[inline]
    pub unsafe fn alloc(&mut self) -> *mut u8 {
        self.reserve_exact(1);
        let index = self.len;
        self.len += 1;
        self.get_unchecked_mut(index)
    }

    #[inline]
    pub unsafe fn push(&mut self, data: *mut u8) {
        self.reserve_exact(1);
        let index = self.len;
        self.len += 1;
        self.initialize_unchecked(index, data);
    }

    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize, ptr: *mut u8) {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);

        let last = self.get_unchecked_mut(self.len - 1);
        let target = self.get_unchecked_mut(index);
        let size = self.item_layout.size();
        std::ptr::copy_nonoverlapping(target, ptr, size);
        std::ptr::copy(last, target, size);
        self.len -= 1;
    }

    #[inline]
    pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);

        let new_len = self.len - 1;
        let size = self.item_layout.size();

        if index != new_len {
            std::ptr::swap_nonoverlapping::<u8>(
                self.get_unchecked_mut(index),
                self.get_unchecked_mut(new_len),
                size
            );
        }

        self.len = new_len;
        
        // Cannot use `Self::get_unchecked_mut`, as the forgotten value is stored out of bounds of
        // this structure at index `self.len`
        self.get_ptr_mut().add(new_len * size)
    }
    
    #[inline]
    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, index: usize) {
        let ptr = self.swap_remove_and_forget_unchecked(index);
        if let Some(drop) = self.drop {
            drop(ptr);
        }
    }

    /// SAFETY:
    /// - `index` is < `self.len`
    /// - `data` is valid
    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, data: *mut u8) {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);
        let size = self.item_layout.size();
        let ptr = self.get_ptr_mut().add(index * size);
        std::ptr::copy_nonoverlapping::<u8>(data, ptr, size);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        if self.item_layout.size() == 0 {
            return;
        }

        let available = self.capacity - self.len;
        if available > additional {
            return;
        }

        let increment = additional - available; 
        unsafe { self.grow_exact(increment); }
    }

    /// SAFETY:
    /// - Should only be called on non-zero sized types
    pub unsafe fn grow_exact(&mut self, increment: usize) {
        debug_assert!(self.item_layout.size() != 0);

        let capacity = self.capacity + increment;
        let array_layout = Self::array_layout(&self.item_layout, capacity);
        let old_array_layout = Self::array_layout(&self.item_layout, self.capacity);
        let data = if self.data.is_null() {
            std::alloc::alloc(array_layout)
        } else {
            std::alloc::realloc(self.get_ptr_mut(), old_array_layout, array_layout.size())
        };

        if data.is_null() {
            handle_alloc_error(array_layout);
        }

        self.data = data;
        self.capacity = capacity;

    }

    pub unsafe fn clear(&mut self) {
        if let Some(drop) = self.drop {
            let size = self.item_layout.size();
            for i in 0..self.len {
                let ptr = self.get_ptr_mut().add(i * size);
                drop(ptr);
            }
        }

        self.len = 0;
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get_ptr(&self) -> *const u8 {
        self.data as *const _
    }

    #[inline]
    pub fn get_ptr_mut(&mut self) -> *mut u8 {
        self.data
    }

    /// SAFETY:
    /// - `index` is < `self.len`
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> *const u8 {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);
        let size = self.item_layout.size();
        self.get_ptr().add(index * size)
    }
    
    /// SAFETY:
    /// - `index` is < `self.len`
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);
        let size = self.item_layout.size();
        self.get_ptr_mut().add(index * size)
    }

    /// SAFETY:
    /// - `index` is < `self.len`
    /// - `data` points to valid data of `self.item_layout` type
    /// - data at `index` has been initialized
    pub unsafe fn replace_unchecked(&mut self, index: usize, data: *mut u8) {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);

        // Drop panic prevention
        let old_len = self.len;
        self.len = 0;
        let ptr = self.get_unchecked_mut(index);

        if let Some(drop) = self.drop {
            // In case drop on `ptr` panics, drop `data` aswell
            struct OnDrop<F: FnMut()>(F);
            impl<F: FnMut()> Drop for OnDrop<F> {
                fn drop(&mut self) {
                    (self.0)();
                }
            }
            
            let on_unwind = OnDrop(|| drop(data));
            drop(ptr);
            std::mem::forget(on_unwind);
        }

        std::ptr::copy_nonoverlapping(data, ptr, self.item_layout.size());
        self.len = old_len;
    }

    fn array_layout(layout: &Layout, n: usize) -> Layout {
        let align = layout.align();
        let size = layout.size();
        let padding_needed = (size.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1)).wrapping_sub(size);
        let padded_size = size + padding_needed;
        let alloc_size = padded_size.checked_mul(n).expect("Layout must be valid!");

        debug_assert!(size == padded_size);

        // SAFETY:
        // - align has been checked to be valid
        // alloc_size has been padded
        unsafe { Layout::from_size_align_unchecked(alloc_size, align) }
    }
}

impl Drop for RawStore {
    fn drop(&mut self) {
        unsafe {
            self.clear();

            let array_layout = Self::array_layout(&self.item_layout, self.capacity());
            if array_layout.size() > 0 {
                std::alloc::dealloc(self.data, array_layout);
            }
        }
    }
}

