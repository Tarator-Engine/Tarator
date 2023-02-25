use crate::{
    archetype::{ ArchetypeId, Archetypes },
    bundle::{ Bundle, Bundles, CloneBundle },
    component::{ ComponentQuery, ComponentQueryMut, Components },
    entity::{ Entities, Entity },
    store::sparse::SparseSetIndex,
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
///     // Or we can implement [`Component`] manually
///     struct Age(u8);
///     impl Component for Age {}
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
///     let (name, age) = world.entity_get::<(Name, Age)>(human);
///     assert!(name.unwrap().0 == String::from("Max Mustermann"));
///     assert!(age.unwrap().0 == 42);
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
///         let whole = world.entity_get::<Whole>(entity).unwrap();
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
    bundles: Bundles,
    components: Components,
    entities: Entities,
}

impl World {
    /// Will panic if it gets called more than [`usize::MAX`] times
    #[inline]
    pub fn new() -> Self {
        Self {
            id: WorldId::new(),
            archetypes: Archetypes::new(),
            bundles: Bundles::new(),
            components: Components::new(),
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
        Self::inner_entity_set(
            entity,
            data, 
            &mut self.archetypes,
            &mut self.bundles,
            &mut self.components,
            &mut self.entities
        )
    }

    /// Unsets a given [`Bundle`] on `entity`.
    ///
    /// Using this function may result in some memory relocations, so calling this often may result
    /// in fairly poor performance.
    #[inline]
    pub fn entity_unset<T: Bundle>(&mut self, entity: Entity) {
        Self::inner_entity_unset::<T>(
            entity,
            &mut self.archetypes,
            &mut self.bundles,
            &mut self.components,
            &mut self.entities
        )
    }

    /// Returns a reference to the [`Component`] data of given [`Entity`]. If the [`Entity`]
    /// doesn't have [`Component`] from the given [`Bundle`], the returned tuple field will be
    /// [`None`].
    #[inline]
    pub fn entity_get<'a, T: Bundle>(&'a self, entity: Entity) -> T::WrappedRef<'a> {
        Self::inner_entity_get::<T>(entity, &self.archetypes, &self.components, &self.entities)
    }

    /// Returns a mutable reference to the [`Component`] data of given [`Entity`]. If the [`Entity`]
    /// doesn't have [`Component`] from the given [`Bundle`], the returned tuple field will be
    /// [`None`].
    #[inline]
    pub fn entity_get_mut<'a, T: Bundle>(&'a mut self, entity: Entity) -> T::WrappedMutRef<'a> {
        Self::inner_entity_get_mut::<T>(entity, &mut self.archetypes, &self.components, &self.entities)
    }

    /// Returns a [`Vec<Entity>`] with every [`Entity`] that has given [`Bundle`].
    pub fn entity_collect<T: Bundle>(&mut self) -> Vec<Entity> {
        Self::inner_entity_collect::<T>(&mut self.archetypes, &mut self.bundles, &mut self.components)
    }

    /// Iterates over every stored [`Bundle`].
    #[inline]
    pub fn component_query<'a, T: Bundle>(&'a mut self) -> ComponentQuery<'a, T> {
        Self::inner_component_query(&mut self.archetypes, &mut self.bundles, &mut self.components)
    }

    /// Iterates mutably over every stored [`Bundle`].
    #[inline]
    pub fn component_query_mut<'a, T: Bundle>(&'a mut self) -> ComponentQueryMut<'a, T> {
        Self::inner_component_query_mut(&mut self.archetypes, &mut self.bundles, &mut self.components)
    }

    /// Clones every [`CloneBundle`] into a [`Vec`]
    #[inline]
    pub fn component_collect<'a, T: CloneBundle>(&mut self) -> Vec<T> {
        Self::inner_component_collect(&mut self.archetypes, &mut self.bundles, &mut self.components)
    }
}

/// ///////////////////////////////////////////////////////////////////////////////////////////////
/// INNER ENTITY FUNCTIONS ////////////////////////////////////////////////////////////////////////
/// ///////////////////////////////////////////////////////////////////////////////////////////////
impl World {
    #[inline]
    fn inner_entity_create(
        archetypes: &mut Archetypes,
        entities: &mut Entities
    ) -> Entity {
        let (entity, entity_meta) = entities.create();
        // SAFETY:
        // - Getting the empty archetype is safe, as it should already exist
        // - We know that we carry the empty archetype with us
        unsafe {
            let archetype = archetypes.get_unchecked_mut(ArchetypeId::EMPTY.index());
            let mut table = archetype.table_lock();
            entity_meta.index = table.len();
            entity_meta.archetype_id = archetype.id();
            table.set_empty(entity);
        }

        entity
    }

    fn inner_entity_destroy(
        entity: Entity,
        archetypes: &mut Archetypes,
        entities: &mut Entities
    ) {
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

        let replaced_entity = unsafe { archetype.table_lock().drop_entity(entity_meta.index) };

        if let Some(replaced_entity) = replaced_entity {
            // Set the index of the moved entity
            //
            // SAFETY:
            // Entity definitely exists
            let replaced_entity_meta = unsafe { entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = entity_meta.index;
        }
    }

    fn inner_entity_set<T: Bundle>(
        entity: Entity,
        data: T,
        archetypes: &mut Archetypes,
        bundles: &mut Bundles,
        components: &mut Components,
        entities: &mut Entities
    ) {
        let entity_meta = entities
            .get_mut(entity)
            .expect(format!("{:#?} is no more valid!", entity).as_str());

        let archetype_id = 'relocate: {
            if entity_meta.is_empty() {
                let bundle_info = bundles.init::<T>(components);

                let archetype = archetypes.get_from_bundle_mut(bundle_info);
                break 'relocate if let Some(archetype) = archetype {
                    archetype.id()
                } else {
                    archetypes.create_with_capacity(bundle_info.components(), components, 1)
                };
            }

            // SAFETY:
            // `entity_meta.archetype_id` is definitely valid
            let archetype_id = unsafe {
                Self::get_add_archetype_id::<T>(
                    archetypes,
                    entity_meta.archetype_id,
                    bundles,
                    components,
                    1,
                )
            };

            // If [`Archetype`] of `entity` does not need to change
            if archetype_id == entity_meta.archetype_id {
                // SAFETY:
                // `self.entities` guarantees that the archetype does exist
                let archetype = unsafe { archetypes.get_unchecked_mut(archetype_id.index()) };

                let ids = T::get_component_ids(components);
                archetype.table_lock().set(&ids, entity_meta.index, data);

                return;
            }

            break 'relocate archetype_id;
        };

        // If [`Archetype`] of `entity` does change
        let (old_archetype, new_archetype) = archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (mut old_table, mut new_table) = (old_archetype.table_lock(), new_archetype.table_lock());
        let (old_index, new_index) = (entity_meta.index, new_table.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        // SAFETY:
        // We initialize the remaining components right after
        let replaced_entity = unsafe { old_table.move_into_parent(&mut new_table, old_index) };
        let ids = T::get_component_ids(components);
        new_table.init(&ids, entity, data);

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
        bundles: &mut Bundles,
        components: &mut Components,
        entities: &mut Entities
    ) {
        let Some(entity_meta) = entities.get_mut(entity) else {
            return;
        };

        if entity_meta.is_empty() {
            return;
        }

        // SAFETY:
        // `entity_meta.archetype_id` is definitely valid
        let archetype_id = unsafe {
            Self::get_sub_archetype_id::<T>(
                archetypes,
                entity_meta.archetype_id,
                bundles,
                components,
                1,
            )
        };

        if archetype_id == entity_meta.archetype_id {
            return;
        }

        // If [`Archetype`] of `entity` does change
        let (old_archetype, new_archetype) = archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (mut old_table, mut new_table) = (old_archetype.table_lock(), new_archetype.table_lock());
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
        archetypes: &Archetypes,
        components: &Components,
        entities: &Entities
    ) -> T::WrappedRef<'a> {
        let Some(meta) = entities.get(entity) else {
            return T::empty_ref();
        };
        let Some(archetype) = archetypes.get(meta.archetype_id) else {
            return T::empty_ref();
        };

        let ids = T::get_component_ids(components);

        let table = archetype.table_lock();
        if table.len() < meta.index {
            return T::empty_ref();
        }

        unsafe { table.get::<T>(&ids, meta.index) }
    }

    #[inline]
    fn inner_entity_get_mut<'a, T: Bundle>(
        entity: Entity,
        archetypes: &mut Archetypes,
        components: &Components,
        entities: &Entities
    ) -> T::WrappedMutRef<'a> {
        let Some(meta) = entities.get(entity) else {
            return T::empty_mut_ref();
        };
        let Some(archetype) = archetypes.get_mut(meta.archetype_id) else {
            return T::empty_mut_ref();
        };

        let ids = T::get_component_ids(components);
        
        let mut table = archetype.table_lock();
        if table.len() < meta.index {
            return T::empty_mut_ref();
        }

        unsafe { table.get_mut::<T>(&ids, meta.index) }
    }

    #[inline]
    fn inner_entity_collect<T: Bundle>(
        archetypes: &mut Archetypes,
        bundles: &mut Bundles,
        components: &mut Components
    ) -> Vec<Entity> {
        let bundle_info = bundles.init::<T>(components);
        let archetype_id = archetypes
            .get_id_from_components_or_create_with_capacity(
                components,
                bundle_info.components(),
                1,
            );
        // SAFETY:
        // [`Archetype`] was just created
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut entities = archetype.table_lock().entities();

        for parent_id in archetype.parents() {
            // SAFETY:
            // Parent definitely exists
            let parent = unsafe { archetypes.get_unchecked(parent_id.index()) };
            entities.append(&mut parent.table_lock().entities());
        }

        entities
    }
}

/// ///////////////////////////////////////////////////////////////////////////////////////////////
/// INNER COMPONENT FUNCTIONS /////////////////////////////////////////////////////////////////////
/// ///////////////////////////////////////////////////////////////////////////////////////////////
impl World {
    #[inline]
    fn inner_component_query<'a, T: Bundle>(
        archetypes: &'a mut Archetypes,
        bundles: &'a mut Bundles,
        components: &'a mut Components,
    ) -> ComponentQuery<'a, T> {
        let bundle_info = bundles.init::<T>(components);
        let archetype_id = archetypes
            .get_id_from_components_or_create_with_capacity(
                components,
                bundle_info.components(),
                1,
            );

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQuery::new(&archetype_ids, archetypes, components)
    }
    
    #[inline]
    fn inner_component_query_mut<'a, T: Bundle>(
        archetypes: &'a mut Archetypes,
        bundles: &'a mut Bundles,
        components: &'a mut Components,
    ) -> ComponentQueryMut<'a, T> {
        let bundle_info = bundles.init::<T>(components);
        let archetype_id = archetypes
            .get_id_from_components_or_create_with_capacity(
                components,
                bundle_info.components(),
                1,
            );

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQueryMut::new(&archetype_ids, archetypes, components)
    }

    #[inline]
    pub fn inner_component_collect<'a, T: CloneBundle>(
        archetypes: &'a mut Archetypes,
        bundles: &'a mut Bundles,
        components: &'a mut Components,
    ) -> Vec<T> {
        let mut result_bundles = Vec::new();
        for bundle in Self::inner_component_query::<T>(archetypes, bundles, components) {
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
    #[inline]
    unsafe fn get_add_archetype_id<T: Bundle>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        bundles: &mut Bundles,
        components: &mut Components,
        on_create_capacity: usize,
    ) -> ArchetypeId {
        let bundle_info = bundles.init::<T>(components);
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = bundle_info.components().clone();
        bundle_components.insert(archetype.table_lock().component_ids().collect());

        archetypes.get_id_from_components_or_create_with_capacity(
            components,
            &bundle_components,
            on_create_capacity,
        )
    }

    /// Returns the [`ArchetypeId`] from given [`ArchetypeId`], which calculates in subtraction what
    /// new archetype it would fit in.
    ///
    /// # Safety
    ///
    /// `archetype_id` needs to point to a valid [`Archetype`]
    #[inline]
    unsafe fn get_sub_archetype_id<T: Bundle>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        bundles: &mut Bundles,
        components: &mut Components,
        on_create_capacity: usize,
    ) -> ArchetypeId {
        let bundle_info = bundles.init::<T>(components);
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = bundle_info.components().clone();
        bundle_components.remove(archetype.table_lock().component_ids().collect());

        archetypes.get_id_from_components_or_create_with_capacity(
            components,
            &bundle_components,
            on_create_capacity,
        )
    }
}

