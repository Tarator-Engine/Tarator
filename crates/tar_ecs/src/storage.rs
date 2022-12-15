use crate::error::EcsError as Error;
use std::{
    alloc::{
        Layout,
    },
    mem::size_of
};

pub(crate) struct Storage {
    len: usize,
    size: usize,
}

impl Storage {
    pub(crate) unsafe fn new(size: usize, len: usize) -> Self {
        Self {
            len,
            size
        }
    }
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn push(&mut self) -> Result<(), Error> {
        Ok(())        
    }
}
