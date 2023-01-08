use std::{
    sync::{Arc, atomic::{ AtomicBool, AtomicUsize, Ordering }, Mutex},
    ops::{ Deref, DerefMut }, marker::PhantomData, mem::size_of, ptr::{null_mut, copy_nonoverlapping}, alloc::{Layout, System, GlobalAlloc}
};
use crate::{
    world::InnerWorld,
    entity::{desc::Desc, EntityId},
    error::EcsError as Error,
    id::*,
    component::Component
};

use super::tuple::{DataUnit, Unit};



// Id::Version is currentVersion
type InfoDesc = Desc;

pub struct StorePtr<C: Component> {
    world: Arc<Mutex<InnerWorld>>,
    entity: EntityId,
    phantom: PhantomData<C>
}

impl<C: Component> StorePtr<C> {
    pub fn lock(&self) -> Result<StorePtrGuard<C>, Error> {
        let Ok(world) = self.world.lock() else {
            return Err(Error::MutexError);
        };
        let desc = world.desc.get(self.entity)?;
        let Some(arche) = world.arche.arche.get(desc.id.get_index()) else {
            return Err(Error::InvalidIndex(desc.id));
        };
        let store = arche.store.clone();

        while store.lock.load(Ordering::Acquire) {}

        let unit = 'getter: {
            let cid = C::id();
            for unit in &arche.units {
                if unit.id == cid {
                    break 'getter unit; 
                } 
            }
            return Err(Error::InvalidIndex(cid));
        };
        
        let index = desc.index + unit.offset;
        unsafe {
            let lock = store.data.add(index + size_of::<C>()).cast::<AtomicBool>();
            while (*lock).compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) == Ok(false) {}
        }
        Ok(StorePtrGuard { store, index, phantom: PhantomData::default() })
    }
}


pub struct StorePtrGuard<C: Component> {
    store: Arc<Store>,
    index: usize,
    phantom: PhantomData<C> 
}
impl<C: Component> Deref for StorePtrGuard<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        while self.store.lock.load(Ordering::Acquire) {}
        unsafe { &*self.store.data.add(self.index).cast::<C>() }
    }
}
impl<C: Component> DerefMut for StorePtrGuard<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        while self.store.lock.load(Ordering::Acquire) {}
        unsafe { &mut *self.store.data.add(self.index).cast::<C>() }
    }
}
impl<C: Component> Drop for StorePtrGuard<C> {
    fn drop(&mut self) {
        while self.store.lock.load(Ordering::Acquire) {}
        // the lock ist located after the data, just cast to AtomicBool
        unsafe { (*self.store.data.add(self.index + size_of::<C>()).cast::<AtomicBool>()).store(false, Ordering::Release); }
    }
}


pub(crate) struct Store {
    data: *mut u8,
    free: Mutex<Vec<usize>>,
    size: usize,
    len: AtomicUsize,
    lock: AtomicBool,
}

impl Store {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            data: null_mut(),
            free: Mutex::new(Vec::new()),
            len: AtomicUsize::new(0),
            size,
            lock: AtomicBool::new(false)
        }
    }
    pub(crate) fn create(&self) -> Result<usize, Error> {
        let Ok(mut free) = self.free.lock() else {
            return Err(Error::MutexError);
        };
        if let Some(free) = free.pop() {
            Ok(free)
        } else {
            while self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) == Ok(false) {}

            let old_len = self.len.load(Ordering::Acquire);
            let new_len = old_len + 1;

            let old_layout = Layout::array::<u8>(old_len * self.size).unwrap();
            let new_layout = Layout::array::<u8>(new_len * self.size).unwrap();

            unsafe {
                let new_data = System.alloc(new_layout);
                copy_nonoverlapping::<u8>(self.data, new_data, old_len * self.size);
                System.dealloc(self.data, old_layout);

                // go around the borrow checker
                // We are checking for accesses in our FatPointers
                (*(self as *const _ as *mut Self)).data = new_data;
            }

            self.len.store(new_len, Ordering::Release);
            self.lock.store(false, Ordering::Release);

            Ok(old_len * self.size)
        }
    }
    pub(crate) fn set(&self, index: usize, offset: usize, data: DataUnit) -> Result<(), Error> {
        if index >= self.len.load(Ordering::Relaxed) * self.size {
            return Err(Error::InvalidIndex(index));
        }
        unsafe {
            let size = data.size();
            let ptr = self.data.add(index + offset);
            let lock = ptr.add(size).cast::<AtomicBool>();
            while (*lock).compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) == Ok(false) {}

            copy_nonoverlapping::<u8>(data.data, ptr, size);

            (*lock).store(false, Ordering::Release);
        }

        Ok(())
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        todo!()
    }
}

