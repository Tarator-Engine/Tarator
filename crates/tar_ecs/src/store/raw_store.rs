use std::alloc::{ Layout, handle_alloc_error };

/// Type erased vector
pub struct RawStore {
    item_layout: Layout,
    capacity: usize,
    len: usize,
    data: *mut u8
}

impl std::fmt::Debug for RawStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawStore")
            .field("item_layout", &self.item_layout)
            .field("capacity", &self.capacity)
            .field("len", &self.len)
            .field("data", &self.data)
            .finish()
    }
}

impl RawStore {
    /// SAFETY:
    /// - Layout must be valid
    pub unsafe fn new(item_layout: Layout) -> Self {
        Self {
            item_layout,
            capacity: (item_layout.size() == 0).then(|| usize::MAX).unwrap_or_else(|| 0),
            len: 0,
            data: (item_layout.size() == 0).then(|| std::ptr::NonNull::<u8>::dangling().as_ptr()).unwrap_or_else(|| std::ptr::null_mut())
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
        self.get_ptr().add(new_len * size)
    }

    /// SAFETY:
    /// - `index` is < `self.len`
    /// - `data` is valid
    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, data: *mut u8) {
        debug_assert!(index < self.len, "Index is out of bounds! ({}>={})", index, self.len);
        let size = self.item_layout.size();
        let ptr = self.get_ptr().add(index * size);
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
        let array_layout = Self::array_layout(&self.item_layout, capacity).unwrap();
        let old_array_layout = Self::array_layout(&self.item_layout, self.capacity).unwrap();

        let data = if self.data.is_null() {
            std::alloc::alloc(array_layout)
        } else {
            std::alloc::realloc(self.get_ptr(), old_array_layout, array_layout.size())
        };

        if data.is_null() {
            handle_alloc_error(array_layout);
        }

        self.data = data;
        self.capacity = capacity;

    }

    pub unsafe fn dealloc(&mut self) {
        self.len = 0;
        
        let layout = Self::array_layout(&self.item_layout, self.capacity()).unwrap();
        if layout.size() > 0 {
            std::alloc::dealloc(self.data, layout)
        }
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
    pub fn get_ptr(&self) -> *mut u8 {
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
        self.get_ptr().add(index * size)
    }

    
    /// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
    fn array_layout(layout: &Layout, n: usize) -> Option<Layout> {
        let (array_layout, offset) = Self::repeat_layout(layout, n)?;
        debug_assert_eq!(layout.size(), offset);
        Some(array_layout)
    }

    // TODO: replace with `Layout::repeat` if/when it stabilizes
    /// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
    fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
        // This cannot overflow. Quoting from the invariant of Layout:
        // > `size`, when rounded up to the nearest multiple of `align`,
        // > must not overflow (i.e., the rounded value must be less than
        // > `usize::MAX`)
        let padded_size = layout.size() + Self::padding_needed_for(layout, layout.align());
        let alloc_size = padded_size.checked_mul(n)?;

        // SAFETY: self.align is already known to be valid and alloc_size has been
        // padded already.
        unsafe {
            Some((
                Layout::from_size_align_unchecked(alloc_size, layout.align()),
                padded_size,
            ))
        }
    }

    /// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
    const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
        let len = layout.size();

        // Rounded up value is:
        //   len_rounded_up = (len + align - 1) & !(align - 1);
        // and then we return the padding difference: `len_rounded_up - len`.
        //
        // We use modular arithmetic throughout:
        //
        // 1. align is guaranteed to be > 0, so align - 1 is always
        //    valid.
        //
        // 2. `len + align - 1` can overflow by at most `align - 1`,
        //    so the &-mask with `!(align - 1)` will ensure that in the
        //    case of overflow, `len_rounded_up` will itself be 0.
        //    Thus the returned padding, when added to `len`, yields 0,
        //    which trivially satisfies the alignment `align`.
        //
        // (Of course, attempts to allocate blocks of memory whose
        // size and padding overflow in the above manner should cause
        // the allocator to yield an error anyway.)

        let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
        len_rounded_up.wrapping_sub(len)
    }
}
