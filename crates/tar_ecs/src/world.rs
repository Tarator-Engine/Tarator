use parking_lot::RwLock;

use crate::{
    archetype::{ArchetypeId, Archetypes},
    bundle::{Bundle, Bundles, CloneBundle},
    callback::Callback,
    component::{ComponentQuery, ComponentQueryMut, Components, Fake},
    entity::{Entities, Entity},
    store::{
        sparse::SparseSetIndex,
        table::{TMut, TRef, Table},
    },
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
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
///
/// # Examples
///
/// Creating [`Entity`] and assigning/getting a [`Component`] to/from it
///
/// ```
/// fn main() {
///     use tar_ecs::prelude::*;
///
///     let mut world = World::new();
///
///     let human = world.entity_create();
///
///     // Every data type can be a [`Component`] using the [`derive_component)`] macro
///     #[derive(Component)]
///     struct Name(String);
///
///     // Or we can implement [`Component`] manually (discouraged)
///     struct Age(u8);
///     unsafe impl Component for Age {}
///
///     {
///         let name = Name(String::from("Max Mustermann"));
///         let age = Age(42);
///
///         // We can just set both [`Component`]s as a tuple (see [`Bundle`])
///         world.entity_set(human, (name, age));
///     }
///
///     // Getting our [`Component`]s like this returns us `(Option<&Name>, Option<&Age>)`
///     let (name, age) = *world.entity_get::<(Name, Age)>(human).unwrap().get();
///     assert!(name.0 == String::from("Max Mustermann"));
///     assert!(age.0 == 42);
///
///     // No need to destroy our [`Entity`], but may be good practice in some scenarios
///     world.entity_destroy(human);
/// }
/// ```
///
/// Creating n [`Entity`]s, assigning a [`Component`] to each, and iterating over both [`Entity`]s
/// and [`Component`]s
///
/// ```
/// use tar_ecs::prelude::*;
///
/// #[derive(Component)]
/// struct Whole(u32);
///
/// #[derive(Component)]
/// struct Odd(u32);
///
/// fn main() {
///     let mut world = World::new();
///
///     for n in 0..42 {
///         let entity = world.entity_create();
///
///         if n % 2 == 0 {
///             let data = Whole(2);
///             world.entity_set(entity, data);
///         } else {
///             let data = Odd(1);
///             world.entity_set(entity, data);
///         }
///     }
///
///     // This iterates only over [`Entity`]s with a `Whole` [`Component`] set
///     for entity in world.entity_collect::<Whole>() {
///         let whole = world.entity_get::<Whole>(entity).unwrap().get();
///         assert!(whole.0 == 2);
///     }
///
///     // This iterates over all `Odd` [`Component`]s
///     for odd in world.component_query::<Odd>() {
///         assert!(odd.0 == 1);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct World {
    id: WorldId,
    archetypes: Archetypes,
    entities: Entities,
}

impl World {
    /// Will panic if it gets called more than [`usize::MAX`] times
    #[inline]
    pub fn new() -> Self {
        unsafe {
            Components::new();
            Bundles::new();
        }
        Self {
            id: WorldId::new(),
            archetypes: Archetypes::new(),
            entities: Entities::new(),
        }
    }

    /// This [`World`]s [`WorldId`]
    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

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
        Self::inner_entity_create(&mut self.archetypes, &mut self.entities)
    }

    /// Destroys an [`Entity`] and drops all of its [`Component`]s, if any. The [`Entity`] variable
    /// of the user should be discarded, as it is no more valid.
    #[inline]
    pub fn entity_destroy(&mut self, entity: Entity) {
        Self::inner_entity_destroy(entity, &mut self.archetypes, &mut self.entities)
    }

    /// Set a given [`Bundle`] on `entity`. This will move `data` into this [`World`]'s storage. If
    /// the [`Entity`] was already destroyed using [`World::entity_destroy`], it will panic.
    ///
    /// Using this function may result in some memory relocations, so calling this often may result
    /// in fairly poor performance.
    ///
    /// # Todo
    ///
    /// Reconsider if this function should panic or not
    #[inline]
    pub fn entity_set<T: Bundle>(&mut self, entity: Entity, data: T) {
        Self::inner_entity_set(entity, data, &mut self.archetypes, &mut self.entities)
    }

    /// Unsets a given [`Bundle`] on `entity`.
    ///
    /// Using this function may result in some memory relocations, so calling this often may result
    /// in fairly poor performance.
    #[inline]
    pub fn entity_unset<T: Bundle>(&mut self, entity: Entity) {
        Self::inner_entity_unset::<T>(entity, &mut self.archetypes, &mut self.entities)
    }

    /// Returns a reference to the [`Component`] data of given [`Entity`]. If the [`Entity`]
    /// doesn't have [`Component`] from the given [`Bundle`], the returned tuple field will be
    /// [`None`].
    #[inline]
    pub fn entity_get<'a, T: Bundle>(&'a self, entity: Entity) -> Option<TRef<'a, T>> {
        Self::inner_entity_get::<T>(entity, &self.archetypes, &self.entities)
    }

    /// Returns a mutable reference to the [`Component`] data of given [`Entity`]. If the [`Entity`]
    /// doesn't have [`Component`] from the given [`Bundle`], the returned tuple field will be
    /// [`None`].
    #[inline]
    pub fn entity_get_mut<'a, T: Bundle>(&'a mut self, entity: Entity) -> Option<TMut<'a, T>> {
        Self::inner_entity_get_mut::<T>(entity, &self.archetypes, &self.entities)
    }

    #[inline]
    pub fn entity_get_table_and_index(
        &self,
        entity: Entity,
    ) -> Option<(Arc<RwLock<Table>>, usize)> {
        Self::inner_entity_get_table_and_index(entity, &self.archetypes, &self.entities)
    }

    /// Returns a [`Vec<Entity>`] with every [`Entity`] that has given [`Bundle`].
    #[inline]
    pub fn entity_collect<T: Bundle>(&mut self) -> Vec<Entity> {
        Self::inner_entity_collect::<T>(&mut self.archetypes)
    }

    #[inline]
    pub fn entity_callback<T: Callback<Fake>>(&mut self, entity: Entity, callback: &mut T) {
        Self::inner_entity_callback::<T>(entity, callback, &mut self.archetypes, &self.entities)
    }

    /// Iterates over every stored [`Bundle`].
    #[inline]
    pub fn component_query<'a, T: Bundle>(&'a mut self) -> ComponentQuery<'a, T> {
        Self::inner_component_query(&mut self.archetypes)
    }

    /// Iterates mutably over every stored [`Bundle`].
    #[inline]
    pub fn component_query_mut<'a, T: Bundle>(&'a mut self) -> ComponentQueryMut<'a, T> {
        Self::inner_component_query_mut(&mut self.archetypes)
    }

    #[inline]
    pub fn component_query_tables(&mut self, name: &'static str) -> Vec<Arc<RwLock<Table>>> {
        Self::inner_component_query_tables(name, &mut self.archetypes)
    }

    /// Clones every [`CloneBundle`] into a [`Vec`]
    #[inline]
    pub fn component_collect<'a, T: CloneBundle>(&mut self) -> Vec<T> {
        Self::inner_component_collect(&mut self.archetypes)
    }
}

/// ///////////////////////////////////////////////////////////////////////////////////////////////
/// INNER ENTITY FUNCTIONS ////////////////////////////////////////////////////////////////////////
/// ///////////////////////////////////////////////////////////////////////////////////////////////
impl World {
    fn inner_entity_create(archetypes: &mut Archetypes, entities: &mut Entities) -> Entity {
        let (entity, entity_meta) = entities.create();
        // SAFETY:
        // - Getting the empty archetype is safe, as it should already exist
        // - We know that we carry the empty archetype with us
        unsafe {
            let archetype = archetypes.get_unchecked_mut(ArchetypeId::EMPTY.index());
            let mut table = archetype.table_write();
            entity_meta.index = table.len();
            entity_meta.archetype_id = archetype.id();
            table.set_empty(entity);
        }

        entity
    }

    fn inner_entity_destroy(entity: Entity, archetypes: &mut Archetypes, entities: &mut Entities) {
        let Some(entity_meta) = entities.destroy(entity) else {
            return;
        };

        if entity_meta.is_empty() {
            return;
        }

        // If [`Archetype`] of `entity` does change
        let archetype = archetypes
            .get_mut(entity_meta.archetype_id)
            .expect(format!("{:#?} is invalid!", entity_meta.archetype_id).as_str());

        let mut table = archetype.table_write();
        let replaced_entity = unsafe { table.drop_entity(entity_meta.index) };

        if let Some(replaced_entity) = replaced_entity {
            // Set the index of the moved entity
            //
            // SAFETY:
            // Entity definitely exists
            let replaced_entity_meta =
                unsafe { entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = entity_meta.index;
        }
    }

    fn inner_entity_set<T: Bundle>(
        entity: Entity,
        data: T,
        archetypes: &mut Archetypes,
        entities: &mut Entities,
    ) {
        let entity_meta = entities
            .get_mut(entity)
            .expect(format!("{:#?} is no more valid!", entity).as_str());

        let archetype_id = 'relocate: {
            if entity_meta.is_empty() {
                let bundle_id = Bundles::init::<T>();

                break 'relocate Bundles::get_bundle(bundle_id, |bundle| {
                    let archetype = archetypes.get_from_bundle(bundle);
                    if let Some(archetype) = archetype {
                        archetype.id()
                    } else {
                        archetypes.create_with_capacity(bundle, 1)
                    }
                });
            }

            // SAFETY:
            // `entity_meta.archetype_id` is definitely valid
            let archetype_id =
                unsafe { Self::get_add_archetype_id::<T>(archetypes, entity_meta.archetype_id, 1) };

            // If [`Archetype`] of `entity` does not need to change
            if archetype_id == entity_meta.archetype_id {
                // SAFETY:
                // `self.entities` guarantees that the archetype does exist
                let archetype = unsafe { archetypes.get_unchecked_mut(archetype_id.index()) };
                archetype.table_write().set(entity_meta.index, data);

                return;
            }

            break 'relocate archetype_id;
        };

        // If [`Archetype`] of `entity` does change
        let (old_archetype, new_archetype) =
            archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (mut old_table, mut new_table) =
            (old_archetype.table_write(), new_archetype.table_write());
        let (old_index, new_index) = (entity_meta.index, new_table.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        // SAFETY:
        // We initialize the remaining components right after
        let replaced_entity = unsafe { old_table.move_into_parent(&mut new_table, old_index) };
        new_table.init(entity, data);

        if let Some(replaced_entity) = replaced_entity {
            // Set the index of the moved entity
            //
            // SAFETY:
            // Entity definitely exists
            let replaced_entity_meta =
                unsafe { entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = old_index;
        }
    }

    pub fn inner_entity_unset<T: Bundle>(
        entity: Entity,
        archetypes: &mut Archetypes,
        entities: &mut Entities,
    ) {
        let Some(entity_meta) = entities.get_mut(entity) else {
            return;
        };

        if entity_meta.is_empty() {
            return;
        }

        // SAFETY:
        // `entity_meta.archetype_id` is definitely valid
        let archetype_id =
            unsafe { Self::get_sub_archetype_id::<T>(archetypes, entity_meta.archetype_id, 1) };

        if archetype_id == entity_meta.archetype_id {
            return;
        }

        // If [`Archetype`] of `entity` does change
        let (old_archetype, new_archetype) =
            archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (mut old_table, mut new_table) =
            (old_archetype.table_write(), new_archetype.table_write());
        let (old_index, new_index) = (entity_meta.index, new_table.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        let replaced_entity = unsafe { old_table.move_into_child(&mut new_table, old_index) };

        if let Some(replaced_entity) = replaced_entity {
            // Set the index of the moved entity
            //
            // SAFETY:
            // Entity definitely exists
            let replaced_entity_meta =
                unsafe { entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = old_index;
        }
    }

    fn inner_entity_get<'a, T: Bundle>(
        entity: Entity,
        archetypes: &'a Archetypes,
        entities: &Entities,
    ) -> Option<TRef<'a, T>> {
        let meta = entities.get(entity)?;
        let archetype = archetypes.get(meta.archetype_id)?;

        let table = archetype.table_read();
        if table.len() < meta.index {
            return None;
        }

        unsafe { Some(TRef::new(table.get::<T>(meta.index)?, table)) }
    }

    fn inner_entity_get_mut<'a, T: Bundle>(
        entity: Entity,
        archetypes: &'a Archetypes,
        entities: &Entities,
    ) -> Option<TMut<'a, T>> {
        let meta = entities.get(entity)?;
        let archetype = archetypes.get(meta.archetype_id)?;

        let mut table = archetype.table_write();
        if table.len() < meta.index {
            return None;
        }

        unsafe { Some(TMut::new(table.get_raw_mut::<T>(meta.index)?, table)) }
    }

    fn inner_entity_get_table_and_index(
        entity: Entity,
        archetypes: &Archetypes,
        entities: &Entities,
    ) -> Option<(Arc<RwLock<Table>>, usize)> {
        let meta = entities.get(entity)?;
        let archetype = archetypes.get(meta.archetype_id)?;

        Some((archetype.table(), meta.index))
    }

    fn inner_entity_collect<T: Bundle>(archetypes: &mut Archetypes) -> Vec<Entity> {
        let id = Bundles::init::<T>();
        let archetype_id = Bundles::get_bundle(id, |bundle| {
            archetypes.get_id_from_components_or_create_with_capacity(bundle, 1)
        });

        // SAFETY:
        // [`Archetype`] was just created
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut entities = archetype.table_read().entities();

        for parent_id in archetype.parents() {
            // SAFETY:
            // Parent definitely exists
            let parent = unsafe { archetypes.get_unchecked(parent_id.index()) };
            entities.append(&mut parent.table_read().entities());
        }

        entities
    }

    fn inner_entity_callback<T: Callback<Fake>>(
        entity: Entity,
        callback: &mut T,
        archetypes: &Archetypes,
        entities: &Entities,
    ) {
        let Some(meta) = entities.get(entity) else {
            return;
        };

        let Some(archetype) = archetypes.get(meta.archetype_id) else {
            return;
        };

        let mut table = archetype.table_write();

        let Some(callback_id) = Components::get_callback_id_from::<T>() else {
            return;
        };

        // SAFETY:
        // We can guarantee that meta index is valid
        unsafe {
            let callback = callback as *mut _ as *mut u8;
            table.foreach_at(meta.index, |id, data| {
                Components::get_info(id, |info| {
                    let Some(func) = info.callback(callback_id) else {
                        return;
                    };

                    func(callback, data)
                })
            });
        }
    }
}

/// ///////////////////////////////////////////////////////////////////////////////////////////////
/// INNER COMPONENT FUNCTIONS /////////////////////////////////////////////////////////////////////
/// ///////////////////////////////////////////////////////////////////////////////////////////////
impl World {
    fn inner_component_query<'a, T: Bundle>(
        archetypes: &'a mut Archetypes,
    ) -> ComponentQuery<'a, T> {
        let id = Bundles::init::<T>();
        let archetype_id = Bundles::get_bundle(id, |bundle| {
            archetypes.get_id_from_components_or_create_with_capacity(bundle, 1)
        });

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQuery::new(&archetype_ids, archetypes)
    }

    fn inner_component_query_mut<'a, T: Bundle>(
        archetypes: &'a mut Archetypes,
    ) -> ComponentQueryMut<'a, T> {
        let id = Bundles::init::<T>();
        let archetype_id = Bundles::get_bundle(id, |bundle| {
            archetypes.get_id_from_components_or_create_with_capacity(bundle, 1)
        });

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQueryMut::new(&archetype_ids, archetypes)
    }

    fn inner_component_query_tables(
        name: &'static str,
        archetypes: &mut Archetypes,
    ) -> Vec<Arc<RwLock<Table>>> {
        let id = Bundles::init_from_name(name);
        let archetype_id = Bundles::get_bundle(id, |bundle| {
            archetypes.get_id_from_components_or_create_with_capacity(bundle, 1)
        });

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        let mut tables = Vec::with_capacity(archetype_ids.len());

        for id in archetype_ids {
            if let Some(archetype) = archetypes.get(id) {
                tables.push(archetype.table())
            } else {
                debug_assert!(false, "Invalid Id was passed!");
            }
        }

        tables
    }

    fn inner_component_collect<'a, T: CloneBundle>(archetypes: &'a mut Archetypes) -> Vec<T> {
        let mut result_bundles = Vec::new();
        for bundle in Self::inner_component_query::<T>(archetypes) {
            result_bundles.push(<T as CloneBundle>::clone(bundle));
        }

        result_bundles
    }

    /// Returns the [`ArchetypeId`] from given [`ArchetypeId`], which calculates in addition what
    /// new archetype it would fit in.
    ///
    /// # Safety
    ///
    /// `archetype_id` needs to point to a valid [`Archetype`]
    unsafe fn get_add_archetype_id<T: Bundle>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        on_create_capacity: usize,
    ) -> ArchetypeId {
        let bundle_id = Bundles::init::<T>();
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = Bundles::get_bundle(bundle_id, |bundle| bundle.clone());
        bundle_components.insert(archetype.table_read().component_ids().collect());

        archetypes
            .get_id_from_components_or_create_with_capacity(&bundle_components, on_create_capacity)
    }

    /// Returns the [`ArchetypeId`] from given [`ArchetypeId`], which calculates in subtraction what
    /// new archetype it would fit in.
    ///
    /// # Safety
    ///
    /// `archetype_id` needs to point to a valid [`Archetype`]
    unsafe fn get_sub_archetype_id<T: Bundle>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        on_create_capacity: usize,
    ) -> ArchetypeId {
        let bundle_id = Bundles::init::<T>();
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = Bundles::get_bundle(bundle_id, |bundle| bundle.clone());
        bundle_components.insert(archetype.table_read().component_ids().collect());

        archetypes
            .get_id_from_components_or_create_with_capacity(&bundle_components, on_create_capacity)
    }
}
