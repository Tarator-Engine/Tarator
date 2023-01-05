use std::{
    alloc::{
        System, GlobalAlloc, Layout
    },
    ptr::copy
};

use crate::{
    error::EcsError as Error,
    component::TupleUnit
};

pub(crate) struct Storage {
    len: usize,
    size: usize,
    data: *mut u8
}

impl Storage {
    pub(crate) unsafe fn new(size: usize, len: usize) -> Result<Self, Error> {
        let layout = Layout::array::<u8>(size * len).unwrap();
        let data = System.alloc_zeroed(layout);
        Ok(Self {
            len,
            size,
            data
        })
    }
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn increase(&mut self) -> Result<(), Error> {
        todo!()
    }
    pub(crate) fn decrease(&mut self) -> Result<(), Error> {
        todo!()
    }
    pub(crate) unsafe fn set(&mut self, index: usize, unit: TupleUnit) -> Result<(), Error> {
        self.check_index(index)?; 
        let data = self.data.add(index * self.size + unit.index);
        copy(unit.data, data, unit.size);
        Ok(())
    }
    pub(crate) fn unset(&mut self, index: usize) -> Result<(), Error> {
        self.check_index(index)?; 
        todo!()
    }
    #[inline]
    fn check_index(&self, index: usize) -> Result<(), Error> {
        if index >= self.len {
            return Err(Error::InvalidIndex(index));
        }
        Ok(())
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        let layout = Layout::array::<u8>(self.size * self.len).unwrap();
        unsafe { System.dealloc(self.data, layout); }
    }
}

