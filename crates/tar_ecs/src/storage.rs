use crate::{
    error::EcsError as Error,
    component::TupleUnit
};

pub(crate) struct Storage {
    len: usize,
    size: usize
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
    pub(crate) fn increase(&mut self) -> Result<(), Error> {
        todo!()
    }
    pub(crate) fn decrease(&mut self) -> Result<(), Error> {
        todo!()
    }
    pub(crate) fn set(&mut self, index: usize, unit: TupleUnit) -> Result<(), Error> {
        self.check_index(index)?; 
        todo!()
    }
    pub(crate) fn unset(&mut self, index: usize) -> Result<(), Error> {
        self.check_index(index)?; 
        todo!()
    }
    fn check_index(&self, index: usize) -> Result<(), Error> {
        if index >= self.len {
            return Err(Error::InvalidIndex(index));
        }
        Ok(())
    }
}

