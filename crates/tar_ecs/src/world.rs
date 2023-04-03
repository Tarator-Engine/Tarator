use crate::{
    bundle::{ Bundle, CloneBundle, BundleId, BundleNames },
    callback::{ CallbackName, Callback, CallbackId, CallbackFunc },
    component::{ Empty, ComponentId, Component, ComponentInfo },
    entity::{ Entities, Entity },
    store::{
        sparse::SparseSetIndex, table::{RowIndexer, ConstRowIndexer, Table, Indexer},
    },
    archetype::{Archetypes, Archetype}, type_info::{Local, TypeInfo}
};

use std::{
    sync::atomic::{ AtomicUsize, Ordering },
    marker::PhantomData,
    mem
};

/// Uniquely identifies a [`World`]. Multiple [`World`]s can also be created from different
/// threads, and they'll still have an unique [`WorldId`].
///
/// # Panics
///
/// Will panic if more than [`usize::MAX`] [`WorldId`]s get created
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct WorldId(usize);

static WORLD_COUNT: AtomicUsize = AtomicUsize::new(0);

impl WorldId {
    /// Will panic if it gets called more than [`usize::MAX`] times
    pub fn new() -> Self {
        WORLD_COUNT
            // Relaxed ordering is sufficient, as we do not do any critical procedures
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                count.checked_add(1)
            })
            .map(WorldId)
            .expect("Too many worlds were created!")
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

impl SparseSetIndex for WorldId {
    #[inline]
    fn from_usize(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    fn as_usize(&self) -> usize {
        self.0
    }
}


pub struct Inner;
pub struct Outer;


/// This is the core structure of an ecs instance. Multiple [`World`] can be created, even from
/// different threads, each with an unique [`WorldId`].
#[derive(Debug)]
pub struct World<TI: TypeInfo, Location> {
    id: WorldId,
    archetypes: Archetypes,
    entities: Entities,
    type_info: TI,
    location: PhantomData<Location>
}

impl<TI: TypeInfo, Location> World<TI, Location> {
    /// This [`World`]s [`WorldId`]
    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }
} 

impl World<Local, Outer> {
    /// Will panic if it gets called more than [`usize::MAX`] times
    #[inline]
    pub fn new() -> Self {
        let mut type_info = Local::new();
        let mut archetypes = unsafe { Archetypes::new() };
        let bundle_id = type_info.init_bundle_from::<Empty>();
        archetypes.try_init(bundle_id, &mut type_info);
        
        Self {
            id: WorldId::new(),
            entities: Entities::new(),
            archetypes,
            type_info,
            location: PhantomData
        }
    }
}

impl<TI: TypeInfo> World<TI, Outer> {
    /// Instantiate an [`Entity`] on this [`World`]. The returned [`Entity`] can be used to assign
    /// [`Component`]s on it using [`World::entity_set`], or again destroyed using
    /// [`World::entity_destroy`].
    ///
    /// # Safety
    ///
    /// Using the returned [`Entity`] on a different [`World`] may work, but this may be undefined
    /// behaviour, and is discouraged.
    #[inline]
    pub fn entity_create(&mut self) -> Entity {
        let (entity, meta) = self.entities.create();
        
        let archetype = self.archetypes.get_mut(BundleId::EMPTY).unwrap();
        let table = archetype.table_mut();
        meta.index = table.len();
        meta.bundle_id = BundleId::EMPTY;
        unsafe { table.push_from(entity, Empty, &self.type_info); }

        entity
    }

    /// Destroys an [`Entity`] and drops all of its [`Component`]s, if any. The [`Entity`] variable
    /// of the user should be discarded, as it is no more valid.
    #[inline]
    pub fn entity_destroy(&mut self, entity: Entity) {
        let Some(meta) = self.entities.destroy(entity) else {
            debug_assert!(false, "Entity Already Destroyed!");
            return;
        };

        let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
        let table = archetype.table_mut();

        let replaced_entity = unsafe { table.drop(meta.index) };
        
        if let Some(r_entity) = replaced_entity {
            let r_meta = unsafe { self.entities.get_unchecked_mut(r_entity) };
            r_meta.index = meta.index;
        }
    }

    /// SAFETY:
    /// - Tuple pairs have to be the same as the ones in name
    /// - Tuple pair name and raw data have to be of exact same component
    pub unsafe fn entity_set_raw(&mut self, entity: Entity, names: BundleNames, data: &[*mut u8]) {
        let Some(meta) = self.entities.get_mut(entity) else {
            return;
        };
        
        let to_set_bundle_id = self.type_info.init_bundle(names);

        let Some(info) = self.type_info.get_bundle_info(meta.bundle_id, |meta_info| {

            self.type_info.get_bundle_info(to_set_bundle_id, |info| {

                // No moving required    
                if info.is_subset(meta_info) {        
                    return None;
                }
                
                let info = meta_info + info;
                Some(info)
            }).unwrap()
        }).unwrap() else {  
              
            // No moving required    
            let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
            let table = archetype.table_mut();
            
            unsafe { table.set(meta.index, names, data, &self.type_info) };
            
            return;
        };

        let bundle_id = self.type_info.insert_bundle(info);
        self.archetypes.try_init(bundle_id, &self.type_info);

        let (old_a, new_a) = self.archetypes.get_2_mut(meta.bundle_id, bundle_id).unwrap();
        let (old_t, new_t) = (old_a.table_mut(), new_a.table_mut());
        let (old_index, new_index) = (meta.index, new_t.len());

        let replaced_entity = unsafe { old_t.move_into(new_t, old_index) };
        
        self.type_info.get_bundle_info(to_set_bundle_id, |info| for new_id in info.component_ids(){
            for old_id in old_t.component_ids() {
                if &old_id == new_id {
                    unsafe { new_t.drop_component(new_index, *new_id); }
                }
            }
        }).unwrap();

        unsafe { new_t.init(new_index, names, data, &self.type_info); }
        
        meta.bundle_id = bundle_id;
        meta.index = new_index;

        if let Some(r_entity) = replaced_entity {
            let r_meta = unsafe { self.entities.get_unchecked_mut(r_entity) };
            r_meta.index = old_index
        }
    }

    /// Set a given [`Bundle`] on `entity`. This will move `data` into this [`World`]'s storage. If
    /// the [`Entity`] was already destroyed using [`World::entity_destroy`], it will panic.
    ///
    /// Using this function may result in some memory relocations, so calling this often may result
    /// in fairly poor performance.
    #[inline]
    pub fn entity_set<T: Bundle>(&mut self, entity: Entity, data: T) {
        let Some(meta) = self.entities.get_mut(entity) else {
            return;
        };
        
        let to_set_bundle_id = self.type_info.init_bundle_from::<T>();

        let Some(info) = self.type_info.get_bundle_info(meta.bundle_id, |meta_info| {

            self.type_info.get_bundle_info(to_set_bundle_id, |info| {
  
                // No moving required    
                if info.is_subset(meta_info) {        
                    return None;
                }
                
                let info = meta_info + info;
                Some(info)
            }).unwrap()
        }).unwrap() else {  
            
            // No moving required    
            let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
            let table = archetype.table_mut();
            
            unsafe { table.set_from(meta.index, data, &self.type_info) };
            
            return;
        };

        let bundle_id = self.type_info.insert_bundle(info);
        self.archetypes.try_init(bundle_id, &self.type_info);

        let (old_a, new_a) = self.archetypes.get_2_mut(meta.bundle_id, bundle_id).unwrap();
        let (old_t, new_t) = (old_a.table_mut(), new_a.table_mut());
        let (old_index, new_index) = (meta.index, new_t.len());

        let replaced_entity = unsafe { old_t.move_into(new_t, old_index) };

        self.type_info.get_bundle_info(to_set_bundle_id, |info| for new_id in info.component_ids() {
            if old_t.contains(*new_id) {
                unsafe { new_t.drop_component(new_index, *new_id); }
            }
        });

        unsafe { new_t.init_from(new_index, data, &self.type_info); }
        
        meta.bundle_id = bundle_id;
        meta.index = new_index;

        if let Some(r_entity) = replaced_entity {
            let r_meta = unsafe { self.entities.get_unchecked_mut(r_entity) };
            r_meta.index = old_index
        }
    }

    pub unsafe fn entity_unset_raw(&mut self, name: BundleNames, entity: Entity) {
        todo!()
    }

    pub fn entity_unset<T: Bundle>(&mut self, entity: Entity) {
        todo!()
    }
}


impl<TI: TypeInfo, Location> World<TI, Location> {
    pub unsafe fn component_query_raw(
        &mut self,
        names: BundleNames,
        mut func: impl FnMut(&mut World<TI, Inner>, ConstRowIndexer)
    ) {
        let querier = self.component_querier(names);
        let o_world = mem::transmute::<_, &mut World<TI, Inner>>(self) as *mut _;
        
        for indexer in querier {
            func(&mut *o_world, indexer)
        }
    }

    pub unsafe fn component_query_raw_mut(
        &mut self,
        names: BundleNames,
        mut func: impl FnMut(&mut World<TI, Inner>, RowIndexer)
    ) {
        let querier = self.component_querier_mut(names);
        let o_world = mem::transmute::<_, &mut World<TI, Inner>>(self) as *mut _;
        
        for indexer in querier {
            func(&mut *o_world, indexer)
        }
    }

    /// Iterates over every stored [`Bundle`].
    #[inline]
    pub fn component_query<T: Bundle>(
        &mut self,
        mut func: impl for<'a> FnMut(&'a mut World<TI, Inner>, T::Ref<'a>)
    ) {
        let bundle_id = self.type_info.init_bundle_from::<T>();

        self.archetypes.try_init(bundle_id, &self.type_info);
        
        let querier: RawComponentQuerier<ConstRowIndexer> = unsafe { RawComponentQuerier::new(&mut self.archetypes, bundle_id) };
        let o_world = unsafe { mem::transmute::<_, &mut World<TI, Inner>>(self) as *mut World<TI, Inner> };

        for indexer in querier {
            let data = unsafe { T::from_components_as_ref(&(*o_world).type_info, &mut |id| {
                indexer.get(id)
            }).unwrap() };
            func(unsafe { &mut *o_world }, data);
        }
    }

    /// Iterates mutably over every stored [`Bundle`].
    #[inline]
    pub fn component_query_mut<T: Bundle>(
        &mut self,
        mut func: impl for<'a> FnMut(&mut World<TI, Inner>, T::Mut<'a>)
    ) {
        let bundle_id = self.type_info.init_bundle_from::<T>();

        self.archetypes.try_init(bundle_id, &self.type_info);
        
        let querier: RawComponentQuerier<RowIndexer> = unsafe { RawComponentQuerier::new(&mut self.archetypes, bundle_id) };
        let o_world = unsafe { mem::transmute::<_, &mut World<TI, Inner>>(self) as *mut World<TI, Inner> };

        for indexer in querier {
            let data = unsafe { T::from_components_as_mut(&(*o_world).type_info, &mut |id| {
                indexer.get(id)
            }).unwrap() };
            func(unsafe { &mut *o_world }, data);
        }
    }

    /// Clones every [`CloneBundle`] into a [`Vec`]
    #[inline]
    pub fn component_collect<T: CloneBundle>(&mut self) -> Vec<T> {
        let mut bundles = Vec::new();
        self.component_query::<T>(|_, bundle| bundles.push(T::clone_bundles(bundle)));

        bundles
    }

    pub unsafe fn component_querier(&mut self, names: BundleNames) -> RawComponentQuerier<ConstRowIndexer> {
        let bundle_id = self.type_info.get_bundle_id(&names).unwrap_or_else(|| self.type_info.init_bundle(names));
        self.archetypes.try_init(bundle_id, &self.type_info);
        RawComponentQuerier::new(&mut self.archetypes, bundle_id)
    }

    pub unsafe fn component_querier_mut(&mut self, names: BundleNames) -> RawComponentQuerier<RowIndexer> {
        let bundle_id = self.type_info.get_bundle_id(&names).unwrap_or_else(|| self.type_info.init_bundle(names));
        self.archetypes.try_init(bundle_id, &self.type_info);
        RawComponentQuerier::new(&mut self.archetypes, bundle_id)
    }
}

impl<TI: TypeInfo, Location> World<TI, Location> {
    #[inline]
    pub unsafe fn entity_get_raw<T>(
        &mut self,
        entity: Entity,
        func: impl for<'a> FnOnce(&'a mut World<TI, Inner>, RowIndexer) -> T
    ) -> Option<T> {
        let meta = self.entities.get(entity)?;

        let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
        let table = archetype.table_mut();
        let indexer = RowIndexer::new(meta.index, table);
        
        Some(func(mem::transmute::<_, &mut World<TI, Inner>>(self), indexer))
    }

    #[inline]
    pub fn entity_get<T: Bundle, U>(
        &self,
        entity: Entity,
        func: impl for <'a> FnOnce(&'a World<TI, Inner>, T::Ref<'a>) -> U
    ) -> Option<U> {
        let meta = self.entities.get(entity)?;

        let archetype = self.archetypes.get(meta.bundle_id).unwrap();
        let table = archetype.table();
        let indexer = unsafe { ConstRowIndexer::new(meta.index, table as *const _ as *mut Table) };

        let bundle = unsafe { T::from_components_as_ref(&self.type_info, &mut |id| indexer.get(id) )? };

        Some(func(unsafe { mem::transmute::<_, &World<TI, Inner>>(self) }, bundle))
    }

    #[inline]
    pub fn entity_get_mut<T: Bundle, U>(
        &mut self,
        entity: Entity,
        func: impl for<'a> FnOnce(&'a mut World<TI, Inner>, T::Mut<'a>) -> U
    ) -> Option<U> {
        let meta = self.entities.get(entity)?;

        let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
        let table = archetype.table_mut();
        let indexer = unsafe { RowIndexer::new(meta.index, table) };

        let bundle = unsafe { T::from_components_as_mut(&self.type_info, &mut |id| indexer.get(id) )? };

        Some(func(unsafe { mem::transmute::<_, &mut World<TI, Inner>>(self) }, bundle))
    }

    pub fn entity_query<T: Bundle>(
        &mut self,
        mut func: impl FnMut(&mut World<TI, Inner>, &Entity)
    ) {
        let bundle_id = self.type_info.init_bundle_from::<T>();

        self.archetypes.try_init(bundle_id, &self.type_info);
        let o_world = unsafe { mem::transmute::<_, *mut World<TI, Inner>>(self) };
        let o_archetype = unsafe { (*o_world).archetypes.get_mut(bundle_id).unwrap() };
        {
            let o_table = o_archetype.table_mut();

            for entity in o_table.entities() {
                    func(unsafe { &mut *o_world }, entity);
            }
        }

        for parent in o_archetype.parents() {
            let archetype = unsafe { (*o_world).archetypes.get_mut(*parent).unwrap() };
            let table = archetype.table_mut();

            for entity in table.entities() {
                func(unsafe { &mut *o_world }, entity);
            }
        }
    }

    /// Returns a [`Vec<Entity>`] with every [`Entity`] that has given [`Bundle`].
    #[inline]
    pub fn entity_collect<T: Bundle>(&mut self) -> Vec<Entity> {
        let o_bundle_id = self.type_info.init_bundle_from::<T>();
        self.archetypes.try_init(o_bundle_id, &self.type_info);

        let o_archetype = self.archetypes.get(o_bundle_id).unwrap();
        let mut entities = o_archetype.table().entities().clone();

        for parent in o_archetype.parents() {
            let archetype = self.archetypes.get(*parent).unwrap();
            let table = archetype.table();
            entities.reserve(table.len());
            entities.extend(table.entities());
        }

        entities
    }

    #[inline]
    pub fn entity_callback<T: Callback<Empty>>(&mut self, entity: Entity, callback: &mut T) {
        let Some(meta) = self.entities.get(entity) else {
            return;
        };

        let callback_id = self.type_info.init_callback_from::<T, Empty>();

        let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
        let table = archetype.table_mut();

        let indexer = unsafe { RowIndexer::new(meta.index, table) };

        for component_id in table.component_ids() {
            self.type_info.get_component_info(component_id, |info| {
                if let Some(callback_fn) = info.get_callback(callback_id) {
                    unsafe {callback_fn(callback as *mut _ as *mut u8, indexer.get(component_id).unwrap()) }
                }
            });
        }
    }

    #[inline]
    pub fn entity_callback_raw(&mut self, entity: Entity, name: CallbackName, callback: *mut u8) {
        let Some(meta) = self.entities.get(entity) else {
            return;
        };

        let callback_id = self.type_info.init_callback(name);

        let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
        let table = archetype.table_mut();

        let indexer = unsafe { RowIndexer::new(meta.index, table) };

        for component_id in table.component_ids() {
            self.type_info.get_component_info(component_id, |info| {
                if let Some(callback_fn) = info.get_callback(callback_id) {
                    unsafe {callback_fn(callback, indexer.get(component_id).unwrap()) }
                }
            });
        }
    }
}

impl<TI: TypeInfo> World<TI, Outer> {
    #[inline]
    pub fn callback_init_raw(&mut self, name: CallbackName) -> CallbackId {
        self.type_info.init_callback(name)
    }

    #[inline]
    pub fn callback_init<T: Callback<Empty>>(&mut self) -> CallbackId {
        self.type_info.init_callback_from::<T, Empty>()
    }

    #[inline]
    pub unsafe fn component_init_raw(&mut self, name: &'static str, info: ComponentInfo) -> ComponentId {
        self.type_info.init_component(name, info)
    }
    
    #[inline]
    pub fn component_init<T: Component>(&mut self) -> ComponentId {
        self.type_info.init_component_from::<T>()
    }

    #[inline]
    pub fn component_id_raw(&self, name: &'static str) -> Option<ComponentId> {
        self.type_info.get_component_id(name)
    }

    #[inline]
    pub fn component_id<T: Component>(&self) -> Option<ComponentId> {
        self.type_info.get_component_id_from::<T>()
    }

    #[inline]
    pub unsafe fn component_add_callback_raw(&mut self, component_id: ComponentId, callback_id: CallbackId, func: CallbackFunc) {
        self.type_info.component_add_callback(component_id, callback_id, func)
    }

    #[inline]
    pub fn component_add_callback<T: Callback<U>, U: Component>(&mut self) {
        self.type_info.component_add_callback_from::<T, U>()
    }
}

#[derive(Debug)]
pub struct RawComponentQuerier<I: Indexer> {
    archetypes: *mut Archetypes,
    archetype: *mut Archetype,
    table: *mut Table,
    parent_index: usize,
    index: usize,
    _phantom: PhantomData<I>
}

impl<I: Indexer> RawComponentQuerier<I> {
    unsafe fn new(archetypes: *mut Archetypes, from: BundleId) -> Self {
        let archetype = (*archetypes).get_mut(from).unwrap();
        Self {
            archetypes,
            archetype,
            table: archetype.table_mut(),
            parent_index: 0,
            index: 0,
            _phantom: PhantomData
        }
    }
}

impl<I: Indexer> Iterator for RawComponentQuerier<I> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
unsafe {
        if self.index == (*self.table).len() {
            let bundle_id = (*self.archetype).parents().get(self.parent_index)?;
            self.table = (*self.archetypes).get_mut(*bundle_id).unwrap().table_mut();
            self.parent_index += 1;
            self.index = 0;

            return self.next();
        }

        let indexer = I::new(self.index, mem::transmute(self.table));
        self.index += 1;

        Some(indexer)
}
    }
}
