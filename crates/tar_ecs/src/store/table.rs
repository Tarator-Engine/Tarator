use std::{
    alloc::Layout,
    ptr
};

use crate::{
    bundle::{BundleId, Bundle},
    component::ComponentId,
    entity::Entity,
    store::{
        raw_store::RawStore,
        sparse::{ SparseSet, MutSparseSet }
    }, type_info::TypeInfo
};


#[derive(Clone, Debug, Eq, PartialEq)]
struct ComponentItem {
    index: usize,
    size: usize,
    drop: Option<unsafe fn(*mut u8)>
}


#[derive(Clone, Debug)]
pub struct Table {
    store: RawStore,
    indexer: SparseSet<ComponentId, ComponentItem>,
    entities: Vec<Entity>
}

impl Table {
    #[inline]
    pub unsafe fn new(bundle_id: BundleId, type_info: &impl TypeInfo) -> Self {

        let mut ids: Vec<_> = type_info.get_bundle_info(bundle_id, |info| info.component_ids().iter().map(|id| *id).collect()).unwrap();

        ids.sort_by(|a, b| {
            debug_assert!(a != b, "Duplicate components not yet supported!");

            let a_u = type_info.get_component_info(*a, |info| info.layout().size()).unwrap();
            let b_u = type_info.get_component_info(*b, |info| info.layout().size()).unwrap();

            b_u.cmp(&a_u)
        });

        let mut indexer = MutSparseSet::new();
        let mut size = 0;
        let mut align = 1;


        for id in ids {
            type_info.get_component_info(id, |info| {
                let layout = info.layout();
                indexer.insert(id, ComponentItem {
                    index: size,
                    size: layout.size(),
                    drop: info.drop()
                });
        
                size += layout.size();
        
                if layout.align() > align {
                    align = layout.align()
                }
            });
        }

        size = (size == 0).then(|| 0).unwrap_or_else(|| (size + 7) & !7);

        let layout = Layout::from_size_align(size, align).unwrap();

        Self {
            store: RawStore::new(layout),
            indexer: indexer.lock(),
            entities: Vec::new()
        }
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn component_ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.indexer.indices()
    }

    pub fn contains(&self, id: ComponentId) -> bool {
        self.indexer.contains(id)
    }

    pub unsafe fn push_from<T: Bundle>(&mut self, entity: Entity, data: T, type_info: &impl TypeInfo) {
        self.entities.push(entity);
        let alloc = self.store.alloc();

        data.get_components(type_info, &mut |id, data| {
            let item = self.indexer.get(id).unwrap();
            ptr::copy(data, alloc.add(item.index), item.size);
        });
    }

    pub unsafe fn set_from<T: Bundle>(&mut self, index: usize, data: T, type_info: &impl TypeInfo) {
        debug_assert!(index < self.len());

        let store_data = self.store.get_unchecked_mut(index);

        data.get_components(type_info, &mut |id, data| {
            let item = self.indexer.get(id).expect("Component not part of table!");

            let dst = store_data.add(item.index);

            if let Some(drop) = item.drop {
                drop(dst)
            }

            if item.size != 0 {
                ptr::copy_nonoverlapping(data, dst, item.size)
            }
        })
    }

    pub unsafe fn init_from<T: Bundle>(&mut self, index: usize, data: T, type_info: &impl TypeInfo) {
        debug_assert!(index < self.len());

        let indexer = RowIndexer::new(index, self);

        data.get_components(type_info, &mut |id, data| {
            let size = indexer.get_size(id).expect("Component not part of table!");
            if size != 0 {
                let dst = indexer.get_unchecked(id);
                ptr::copy_nonoverlapping(data, dst, size);
            }
        })
    }
    
    pub unsafe fn move_into(&mut self, other: &mut Self, index: usize) -> Option<Entity> {
        let src_data = self.store.swap_remove_and_forget_unchecked(index);
        let dst_data = other.store.alloc();

        for (src_id, src_item) in self.indexer.iter() {
            if src_item.size != 0 {
                let src = src_data.add(src_item.index);
                
                if let Some(dst_item) = other.indexer.get(*src_id) {
                    debug_assert!(src_item.size == dst_item.size);
                    
                    let dst = dst_data.add(dst_item.index);

                    ptr::copy_nonoverlapping(src, dst, src_item.size)
                } else if let Some(drop) = src_item.drop {
                    drop(src)
                }
            }
        }

        let is_last = self.entities.len() - 1 == index;
        if is_last {
            let entity = self.entities.pop()?;
            other.entities.push(entity);

            None
        } else {
            let entity = self.entities.swap_remove(index);
            other.entities.push(entity);
            
            Some(unsafe { *self.entities.get_unchecked(index) })
        }
    }

    pub unsafe fn drop_component(&mut self, index: usize, id: ComponentId) {
        debug_assert!(index < self.len());
        
        let item = self.indexer.get(id).expect("Component not part of table!");
        let src_data = self.store.get_unchecked_mut(index);

        if let Some(drop) = item.drop {
            let data = src_data.add(item.index);
            drop(data);
        }
    }

    pub unsafe fn drop(&mut self, index: usize) -> Option<Entity> {
        let dealloc = self.store.swap_remove_and_forget_unchecked(index);

        for item in self.indexer.values() {
            if let Some(drop) = item.drop {
                // TODO: Use after free when panicing possible
                drop(dealloc.add(item.index))
            }
        }

        let is_last = self.entities.len() - 1 == index;
        if is_last {
            self.entities.pop()
        } else {
            Some(self.entities.swap_remove(index))
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[inline]
    pub unsafe fn free_unused(&mut self) {
        self.store.free_unused()
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        while self.store.len() > 0 {
            unsafe {
                let data = self.store.swap_remove_and_forget_unchecked(self.store.len() - 1);
                for (_, item) in self.indexer.iter() {
                    if let Some(drop) = item.drop {
                        drop(data.add(item.index))
                    }
                }
            }
        }
        unsafe { self.store.dealloc() }
    }
}

unsafe impl Send for Table {}
// May be careless, but that's what I'm right now anyway, soo...
unsafe impl Sync for Table {}


pub trait Indexer: Sized {
    type Output;

    fn get(&self, id: ComponentId) -> Option<Self::Output>;
    unsafe fn get_unchecked(&self, id: ComponentId) -> Self::Output;
    fn get_size(&self, id: ComponentId) -> Option<usize>;
    fn table<'a>(&'a self) -> &'a Table;
}

pub struct RowIndexer {
    table: *mut Table,
    row: usize
}

impl RowIndexer {
    #[inline]
    pub unsafe fn new(row: usize, table: *mut Table) -> Self {
        debug_assert!(row < (*table).store.len());

        Self { table, row }
    }
}

impl Indexer for RowIndexer {
    type Output = *mut u8;

    #[inline]
    fn get(&self, id: ComponentId) -> Option<Self::Output> {
        unsafe {
            let data = (*self.table).store.get_unchecked_mut(self.row);
            let item = (*self.table).indexer.get(id)?;
            Some(data.add(item.index))
        }
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: ComponentId) -> Self::Output {
        unsafe {
            let data = (*self.table).store.get_unchecked_mut(self.row);
            let item = (*self.table).indexer.get(id).unwrap();
            data.add(item.index)
        }
    }

    #[inline]
    fn get_size(&self, id: ComponentId) -> Option<usize> {
        unsafe { (*self.table).indexer.get(id).map(|item| item.size) }
    }

    fn table<'a>(&'a self) -> &'a Table {
        unsafe { &*self.table }
    }
}

pub struct ConstRowIndexer {
    table: *const Table,
    row: usize
}

impl ConstRowIndexer {
    #[inline]
    pub unsafe fn new(row: usize, table: *const Table) -> Self {
        debug_assert!(row < (*table).store.len());

        Self { table, row }
    }
}

impl Indexer for ConstRowIndexer {
    type Output = *const u8;

    #[inline]
    fn get(&self, id: ComponentId) -> Option<Self::Output> {
        unsafe {
            let data = (*self.table).store.get_unchecked(self.row);
            let item = (*self.table).indexer.get(id)?;
            Some(data.add(item.index))
        }
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: ComponentId) -> Self::Output {
        unsafe {
            let data = (*self.table).store.get_unchecked(self.row);
            let item = (*self.table).indexer.get(id).unwrap();
            data.add(item.index)
        }
    }

    #[inline]
    fn get_size(&self, id: ComponentId) -> Option<usize> {
        unsafe { (*self.table).indexer.get(id).map(|item| item.size) }
    }

    fn table<'a>(&'a self) -> &'a Table {
        unsafe { &*self.table }
    }
}
