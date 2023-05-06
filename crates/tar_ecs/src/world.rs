use crate::{
    bundle::{ Bundle, CloneBundle, BundleId },
    callback::{ Callback, CallbackId },
    component::{ ComponentId, Component },
    entity::{ Entities, Entity },
    store::{
        sparse::SparseSetIndex,
        table::{ RowIndexer, ConstRowIndexer, Table, Indexer },
    },
    archetype::Archetypes,
    type_info::{ Local, TypeInfo }, query::{Query, QueryMut}
};

use std::sync::atomic::{ AtomicUsize, Ordering };

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


/// This is the core structure of an ecs instance. Multiple [`World`] can be created, even from
/// different threads, each with an unique [`WorldId`].
#[derive(Debug)]
pub struct World<TI: TypeInfo> {
    pub(crate) id: WorldId,
    pub(crate) archetypes: Archetypes,
    pub(crate) entities: Entities,
    pub(crate) type_info: TI
}

impl<TI: TypeInfo> World<TI> {
    /// This [`World`]s [`WorldId`]
    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

    #[inline]
    pub fn component_id<T: Component>(&self) -> Option<ComponentId> {
        self.type_info.get_component_id_from::<T>()
    }

    #[inline]
    pub fn callback_init<T: Callback<()>>(&mut self) -> CallbackId {
        self.type_info.init_callback_from::<T, ()>()
    }

    #[inline]
    pub fn component_init<T: Component>(&mut self) -> ComponentId {
        self.type_info.init_component_from::<T>()
    }

    #[inline]
    pub fn component_add_callback<T: Callback<U>, U: Component>(&mut self) {
        self.type_info.component_add_callback_from::<T, U>()
    }
}


impl World<Local> {
    /// Will panic if it gets called more than [`usize::MAX`] times
    #[inline]
    pub fn new() -> Self {
        let mut type_info = Local::new();
        let mut archetypes = unsafe { Archetypes::new() };
        let bundle_id = type_info.init_bundle_from::<()>();
        archetypes.try_init(bundle_id, &mut type_info);
        
        Self {
            id: WorldId::new(),
            entities: Entities::new(),
            archetypes,
            type_info
        }
    }
}

impl<TI: TypeInfo> World<TI> {
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
        unsafe { table.push_from(entity, (), &self.type_info); }

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

    pub fn entity_unset<T: Bundle>(&mut self, entity: Entity) {
        let Some(meta) = self.entities.get_mut(entity) else {
            return;
        };

        let to_unset_bundle_id = self.type_info.init_bundle_from::<T>();
        
        let Some(info) = self.type_info.get_bundle_info(meta.bundle_id, |meta_info| {
            self.type_info.get_bundle_info(to_unset_bundle_id, |info| {
                let info = meta_info - info;
                
                if info.len() == 0 {
                    None
                } else {
                    Some(info)
                }
            }).unwrap() 
        }).unwrap() else {
            return;
        };

        let bundle_id = self.type_info.insert_bundle(info);
        self.archetypes.try_init(bundle_id, &self.type_info);

        let (old_a, new_a) = self.archetypes.get_2_mut(meta.bundle_id, bundle_id).unwrap();
        let (old_t, new_t) = (old_a.table_mut(), new_a.table_mut());
        let (old_index, new_index) = (meta.index, new_t.len());

        let replaced_entity = unsafe { old_t.move_into(new_t, old_index) };

        meta.bundle_id = bundle_id;
        meta.index = new_index;

        if let Some(r_entity) = replaced_entity {
            let r_meta = unsafe { self.entities.get_unchecked_mut(r_entity) };
            r_meta.index = old_index
        }

    }
}

impl<TI: TypeInfo> World<TI> {
    #[inline]
    pub fn entity_get<T: Bundle, U>(
        &self,
        entity: Entity,
        func: impl for <'a> FnOnce(T::Ref<'a>) -> U
    ) -> Option<U> {
        let meta = self.entities.get(entity)?;

        let archetype = self.archetypes.get(meta.bundle_id).unwrap();
        let table = archetype.table();
        let indexer = unsafe { ConstRowIndexer::new(meta.index, table as *const _ as *mut Table) };

        let bundle = unsafe { T::from_components_as_ref(&self.type_info, &mut |id| indexer.get(id) )? };

        Some(func(bundle))
    }

    #[inline]
    pub fn entity_get_mut<T: Bundle, U>(
        &mut self,
        entity: Entity,
        func: impl for<'a> FnOnce(T::Mut<'a>) -> U
    ) -> Option<U> {
        let meta = self.entities.get(entity)?;

        let archetype = self.archetypes.get_mut(meta.bundle_id).unwrap();
        let table = archetype.table_mut();
        let indexer = unsafe { RowIndexer::new(meta.index, table) };

        let bundle = unsafe { T::from_components_as_mut(&self.type_info, &mut |id| indexer.get(id) )? };

        Some(func(bundle))
    }

    /// SAFETY:
    /// Calls to `entity_set`, `entity_unset`, or any component-prefixed function may result in
    /// undefined behaviour
    pub unsafe fn entity_query<T: Bundle>(
        &mut self,
        mut func: impl FnMut(&mut World<TI>, &Entity)
    ) {
        let bundle_id = self.type_info.init_bundle_from::<T>();

        self.archetypes.try_init(bundle_id, &self.type_info);
        let o_world: *mut Self = &mut *self;

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
    pub fn entity_callback<T: Callback<()>>(&mut self, entity: Entity, callback: &mut T) {
        let Some(meta) = self.entities.get(entity) else {
            return;
        };

        let callback_id = self.type_info.init_callback_from::<T, ()>();

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
}

impl<TI: TypeInfo> World<TI> {
    /// Iterates over every stored [`Bundle`].
    #[inline]
    pub fn component_query<T: Bundle>(
        &mut self,
        mut func: impl for<'a> FnMut(T::Ref<'a>)
    ) {
        let bundle_id = self.type_info.init_bundle_from::<T>();

        self.archetypes.try_init(bundle_id, &self.type_info);
        let root = self.archetypes.get(bundle_id).unwrap();

        #[inline]
        fn query<T: Bundle>(type_info: &impl TypeInfo, table: &Table, func: &mut impl for<'a> FnMut(T::Ref<'a>)) {
            for n in 0..table.len() {
                let indexer = unsafe { ConstRowIndexer::new(n, table as *const _ as *mut _) };
                let data = unsafe {
                    T::from_components_as_ref(type_info, &mut |id| {
                        indexer.get(id)
                    }).unwrap()
                };
                func(data)
            }
        }

        query::<T>(&self.type_info, root.table(), &mut func);

        for parent in root.parents() {
            let archetype = self.archetypes.get(*parent).unwrap();
            query::<T>(&self.type_info, archetype.table(), &mut func)
        }
    }

    /// Iterates mutably over every stored [`Bundle`].
    #[inline]
    pub fn component_query_mut<T: Bundle>(
        &mut self,
        mut func: impl for<'a> FnMut(T::Mut<'a>)
    ) {
        let bundle_id = self.type_info.init_bundle_from::<T>();

        self.archetypes.try_init(bundle_id, &self.type_info);
        let world: *mut Self = &mut *self;
        let root = unsafe { (*world).archetypes.get_mut(bundle_id).unwrap() };
            

        #[inline]
        fn query<T: Bundle>(type_info: &impl TypeInfo, table: &mut Table, func: &mut impl for<'a> FnMut(T::Mut<'a>)) {
            for n in 0..table.len() {
                let indexer = unsafe { RowIndexer::new(n, table) };
                let data = unsafe {
                    T::from_components_as_mut(type_info, &mut |id| {
                        indexer.get(id)
                    }).unwrap()
                };
                func(data)
            }
        }

        query::<T>(unsafe { &(*world).type_info }, root.table_mut(), &mut func);

        for parent in root.parents() {
            let archetype = unsafe { (*world).archetypes.get_mut(*parent).unwrap() };
            query::<T>(unsafe { &(*world).type_info }, archetype.table_mut(), &mut func)
        }
    }

    /// Clones every [`CloneBundle`] into a [`Vec`]
    #[inline]
    pub fn component_collect<T: CloneBundle>(&mut self) -> Vec<T> {
        let mut bundles = Vec::new();
        self.component_query::<T>(|bundle| bundles.push(T::clone_bundles(bundle)));

        bundles
    }

    #[inline]
    pub fn get_component_query<T: Bundle>(&mut self) -> Query<T, TI> {
        let bundle_id = self.type_info.init_bundle_from::<T>();
        self.archetypes.try_init(bundle_id, &self.type_info);

        Query::new(bundle_id, self)
    }

    #[inline]
    pub fn get_component_query_mut<T: Bundle>(&mut self) -> QueryMut<T, TI> {
        let bundle_id = self.type_info.init_bundle_from::<T>();
        self.archetypes.try_init(bundle_id, &self.type_info);

        QueryMut::new(bundle_id, self)
    }
}

impl<TI: TypeInfo> World<TI> {
    /// Not recommended to call if:
    /// - A: You're creating a lot of new Entities
    /// - B: You'll be changing a lot of component sets
    ///
    /// May result in rather fragmented heap
    #[inline]
    pub fn free_unused_memory(&mut self) {
        for (_, archetype) in self.archetypes.iter_mut() {
            let table = archetype.table_mut();
            unsafe { table.free_unused(); }
        }
    }
}

