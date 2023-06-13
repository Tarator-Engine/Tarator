use std::{ ptr, marker::PhantomData };

use crate::{
    world::World,
    type_info::TypeInfo,
    bundle::{ Bundle, BundleId },
    store::table::{ Indexer, RowIndexer, ConstRowIndexer, Table }
};


pub struct Query<'a, T: Bundle, TI: TypeInfo> {
    world: &'a World<TI>,
    table: *const Table,
    bundle_ids: Vec<BundleId>,
    index: usize,
    _phantom: PhantomData<T::Ref<'a>>
}

impl<'a, T: Bundle, TI: TypeInfo> Query<'a, T, TI> {
    pub fn new(bundle_id: BundleId, world: &'a World<TI>) -> Self {
        let mut bundle_ids = world.archetypes.get(bundle_id).unwrap().parents().clone();
        bundle_ids.push(bundle_id);

        Self {
            world,
            bundle_ids,
            table: ptr::null(),
            index: 0,
            _phantom: PhantomData
        }
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Iterator for Query<'a, T, TI> {
    type Item = T::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.table.is_null() || self.index >= unsafe { (*self.table).len() } {
            self.index = 0;
            self.table = self.world.archetypes.get(self.bundle_ids.pop()?)?.table();
        }

        let indexer = unsafe { ConstRowIndexer::new(self.index, self.table) };
        self.index += 1;

        return unsafe { T::from_components_as_ref(&*(*self.world).type_info.get(), &mut |id| indexer.get(id) ) };
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Clone for Query<'a, T, TI> {
    fn clone(&self) -> Self {
        Self {
            bundle_ids: self.bundle_ids.clone(),
            ..*self
        } 
    }
}


pub struct QueryMut<'a, T: Bundle, TI: TypeInfo> {
    world: &'a mut World<TI>,
    bundle_ids: Vec<BundleId>,
    table: *mut Table,
    index: usize,
    _phantom: PhantomData<T::Mut<'a>>
}

impl<'a, T: Bundle, TI: TypeInfo> QueryMut<'a, T, TI> {
    pub fn new(bundle_id: BundleId, world: &'a mut World<TI>) -> Self {
        let mut bundle_ids = world.archetypes.get_mut(bundle_id).unwrap().parents().clone();
        bundle_ids.push(bundle_id);

        Self {
            world,
            bundle_ids,
            table: ptr::null_mut(), 
            index: 0,
            _phantom: PhantomData
        }
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Iterator for QueryMut<'a, T, TI> {
    type Item = T::Mut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.table.is_null() || self.index >= unsafe { (*self.table).len() } {
            self.index = 0;
            self.table = self.world.archetypes.get_mut(self.bundle_ids.pop()?)?.table_mut();
        }

        let indexer = unsafe { RowIndexer::new(self.index, self.table) };
        self.index += 1;

        return unsafe { T::from_components_as_mut(&*(*self.world).type_info.get(), &mut |id| indexer.get(id) ) };
    }
}

