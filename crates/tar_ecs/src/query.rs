use std::marker::PhantomData;

use crate::{
    world::World,
    type_info::TypeInfo,
    bundle::{ Bundle, BundleId },
    store::table::{ Indexer, RowIndexer, ConstRowIndexer }
};


pub struct Query<'a, T: Bundle, TI: TypeInfo> {
    world: *const World<TI>,
    bundle_ids: Vec<BundleId>,
    bundle_id: BundleId,
    index: usize,
    _phantom: PhantomData<T::Ref<'a>>
}

impl<'a, T: Bundle, TI: TypeInfo> Query<'a, T, TI> {
    pub fn new(bundle_id: BundleId, world: *const World<TI>) -> Self {
        Self {
            world,
            bundle_ids: unsafe { (*world).archetypes.get(bundle_id).unwrap().parents().clone() },
            bundle_id,
            index: 0,
            _phantom: PhantomData
        }
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Iterator for Query<'a, T, TI> {
    type Item = T::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut table = unsafe { (*self.world).archetypes.get(self.bundle_id)?.table() };
        
        while self.index >= table.len() {
            self.index = 0;
            self.bundle_id = self.bundle_ids.pop()?; 
            table = unsafe { (*self.world).archetypes.get(self.bundle_id)?.table() };
        }

        let indexer = unsafe { ConstRowIndexer::new(self.index, table) };
        self.index += 1;

        return unsafe { T::from_components_as_ref(&(*self.world).type_info, &mut |id| {
            indexer.get(id)
        }) };
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Clone for Query<'a, T, TI> {
    fn clone(&self) -> Self {
        Self {
            world: self.world,
            bundle_ids: self.bundle_ids.clone(),
            bundle_id: self.bundle_id,
            index: self.index,
            _phantom: PhantomData
        } 
    }
}


pub struct QueryMut<'a, T: Bundle, TI: TypeInfo> {
    world: *mut World<TI>,
    bundle_ids: Vec<BundleId>,
    bundle_id: BundleId,
    index: usize,
    _phantom: PhantomData<T::Mut<'a>>
}

impl<'a, T: Bundle, TI: TypeInfo> QueryMut<'a, T, TI> {
    pub fn new(bundle_id: BundleId, world: *mut World<TI>) -> Self {
        Self {
            world,
            bundle_ids: unsafe { (*world).archetypes.get(bundle_id).unwrap().parents().clone() },
            bundle_id,
            index: 0,
            _phantom: PhantomData
        }
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Iterator for QueryMut<'a, T, TI> {
    type Item = T::Mut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut table = unsafe { (*self.world).archetypes.get_mut(self.bundle_id)?.table_mut() };
        
        while self.index >= table.len() {
            self.index = 0;
            self.bundle_id = self.bundle_ids.pop()?; 
            table = unsafe { (*self.world).archetypes.get_mut(self.bundle_id)?.table_mut() };
        }

        let indexer = unsafe { RowIndexer::new(self.index, table) };
        self.index += 1;

        return unsafe { T::from_components_as_mut(&(*self.world).type_info, &mut |id| {
            indexer.get(id)
        }) };
    }
}

impl<'a, T: Bundle, TI: TypeInfo> Clone for QueryMut<'a, T, TI> {
    fn clone(&self) -> Self {
        Self {
            world: self.world,
            bundle_ids: self.bundle_ids.clone(),
            bundle_id: self.bundle_id,
            index: self.index,
            _phantom: PhantomData
        }
    }
}

