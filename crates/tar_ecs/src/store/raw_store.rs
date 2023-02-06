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
        f.debug_struct("BlobVec")
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
            store.init_self_with_capacity(capacity);

            store
        }
    }

    /// SAFETY:
    /// - Should not be called on zero-sized items
    /// - `self.capacity` should equal 0
    pub unsafe fn init_self_with_capacity(&mut self, capacity: usize) {
        debug_assert!(self.item_layout.size() != 0);
        debug_assert!(self.capacity == 0);
    
        let array_layout = Self::array_layout(&self.item_layout, capacity);
        let data = std::alloc::alloc(array_layout);

        if data.is_null() {
            handle_alloc_error(array_layout);
        }

        self.data = data;
        self.capacity = capacity;
    }

    #[inline]
    pub unsafe fn push(&mut self, data: *mut u8) {
        self.reserve_exact(1);
        let index = self.len;
        self.len += 1;
        self.initialize_unchecked(index, data);
    }

    /// SAFETY:
    /// - `index` is < `self.len`
    /// - `data` is valid
    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, data: *mut u8) {
        debug_assert!(index < self.len);
        let ptr = self.get_unchecked_mut(index);
        let size = self.item_layout.size();
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
        let data = std::alloc::realloc(self.get_ptr_mut(), old_array_layout, array_layout.size());

        if data.is_null() {
            handle_alloc_error(array_layout);
        }

        self.data = data;
        self.capacity = capacity;

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
        debug_assert!(index < self.len);
        let size = self.item_layout.size();
        self.get_ptr().add(index * size)
    }
    
    /// SAFETY:
    /// - `index` is < `self.len`
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        let size = self.item_layout.size();
        self.get_ptr_mut().add(index * size)
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

