use crate::{
    archetype::{ArchetypeId, Archetypes},
    bundle::{Bundle, Bundles},
    component::{ComponentQuery, ComponentQueryMut, Components},
    entity::{Entities, Entity},
    store::sparse::SparseSetIndex,
};
use std::sync::atomic::{AtomicUsize, Ordering};

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
///     for entity in world.entity_query::<Whole>() {
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
        let (entity, entity_meta) = self.entities.create();
        // SAFETY:
        // - Getting the empty archetype is safe, as it should already exist
        // - We know that we carry the empty archetype with us
        unsafe {
            let archetype = self.archetypes.get_unchecked_mut(0);
            entity_meta.index = archetype.len();
            entity_meta.archetype_id = archetype.id();
            archetype.set_empty(entity);
        }

        entity
    }

    /// Destroys an [`Entity`] and drops all of its [`Component`]s, if any. The [`Entity`] variable
    /// of the user should be discarded, as it is no more valid.
    #[inline]
    pub fn entity_destroy(&mut self, entity: Entity) {
        let Some(entity_meta) = self.entities.destroy(entity) else {
            return;
        };

        if entity_meta.is_empty() {
            return;
        }

        // If [`Archetype`] of `entity` does change
        let archetype = self
            .archetypes
            .get_mut(entity_meta.archetype_id)
            .expect(format!("{:#?} is invalid!", entity_meta.archetype_id).as_str());

        let replaced_entity = unsafe { archetype.drop_entity(entity_meta.index) };

        if let Some(replaced_entity) = replaced_entity {
            // Set the index of the moved entity
            //
            // SAFETY:
            // Entity definitely exists
            let replaced_entity_meta = unsafe {
                self.entities
                    .get_unchecked_mut(replaced_entity.id() as usize)
            };
            replaced_entity_meta.index = entity_meta.index;
        }
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
    pub fn entity_set<T: Bundle>(&mut self, entity: Entity, data: T) {
        let Self {
            archetypes,
            bundles,
            components,
            entities,
            ..
        } = self;

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
                archetype.set(components, entity_meta.index, data);
                return;
            }

            break 'relocate archetype_id;
        };

        // If [`Archetype`] of `entity` does change
        let (old_archetype, new_archetype) =
            archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (old_index, new_index) = (entity_meta.index, new_archetype.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        // SAFETY:
        // We initialize the remaining components right after
        let replaced_entity = unsafe { old_archetype.move_into_parent(new_archetype, old_index) };
        new_archetype.init(components, entity, data);

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

    /// Unsets a given [`Bundle`] on `entity`.
    ///
    /// Using this function may result in some memory relocations, so calling this often may result
    /// in fairly poor performance.
    #[inline]
    pub fn entity_unset<T: Bundle>(&mut self, entity: Entity) {
        let Self {
            archetypes,
            bundles,
            components,
            entities,
            ..
        } = self;

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
        let (old_archetype, new_archetype) =
            archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (old_index, new_index) = (entity_meta.index, new_archetype.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        let replaced_entity = unsafe { old_archetype.move_into_child(new_archetype, old_index) };

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

    /// Returns a reference to the [`Component`] data of given [`Entity`]. If the [`Entity`]
    /// doesn't have [`Component`] from the given [`Bundle`], the returned tuple field will be
    /// [`None`].
    #[inline]
    pub fn entity_get<'a, T: Bundle>(&self, entity: Entity) -> T::WrappedRef<'a> {
        let Some(meta) = self.entities.get(entity) else {
            return T::empty_ref();
        };
        let Some(archetype) = self.archetypes.get(meta.archetype_id) else {
            return T::empty_ref();
        };
        archetype.get::<T>(&self.components, meta.index)
    }

    /// Returns a mutable reference to the [`Component`] data of given [`Entity`]. If the [`Entity`]
    /// doesn't have [`Component`] from the given [`Bundle`], the returned tuple field will be
    /// [`None`].
    #[inline]
    pub fn entity_get_mut<'a, T: Bundle>(&mut self, entity: Entity) -> T::WrappedMutRef<'a> {
        let Some(meta) = self.entities.get(entity) else {
            return T::empty_mut_ref();
        };
        let Some(archetype) = self.archetypes.get_mut(meta.archetype_id) else {
            return T::empty_mut_ref();
        };

        archetype.get_mut::<T>(&self.components, meta.index)
    }

    /// Returns a [`Vec<Entity>`] with every [`Entity`] that has given [`Bundle`].
    pub fn entity_query<T: Bundle>(&mut self) -> Vec<Entity> {
        let bundle_info = self.bundles.init::<T>(&mut self.components);
        let archetype_id = self
            .archetypes
            .get_id_from_components_or_create_with_capacity(
                &self.components,
                bundle_info.components(),
                1,
            );
        // SAFETY:
        // [`Archetype`] was just created
        let archetype = unsafe { self.archetypes.get_unchecked(archetype_id.index()) };
        let mut entities: Vec<_> = archetype.entities().map(|entity| *entity).collect();

        for parent_id in archetype.parents() {
            // SAFETY:
            // Parent definitely exists
            let parent = unsafe { self.archetypes.get_unchecked(parent_id.index()) };
            entities.append(&mut parent.entities().map(|entity| *entity).collect());
        }

        entities
    }

    /// Iterates over every stored [`Bundle`].
    #[inline]
    pub fn component_query<'a, T: Bundle>(&'a mut self) -> ComponentQuery<'a, T> {
        let bundle_info = self.bundles.init::<T>(&mut self.components);
        let archetype_id = self
            .archetypes
            .get_id_from_components_or_create_with_capacity(
                &self.components,
                bundle_info.components(),
                1,
            );

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { self.archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQuery::new(archetype_ids, &self.archetypes, &self.components)
    }

    /// Iterates mutably over every stored [`Bundle`].
    #[inline]
    pub fn component_query_mut<'a, T: Bundle>(&'a mut self) -> ComponentQueryMut<'a, T> {
        let bundle_info = self.bundles.init::<T>(&mut self.components);
        let archetype_id = self
            .archetypes
            .get_id_from_components_or_create_with_capacity(
                &self.components,
                bundle_info.components(),
                1,
            );

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { self.archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQueryMut::new(archetype_ids, &mut self.archetypes, &self.components)
    }

    /// Returns the [`ArchetypeId`] from given [`ArchetypeId`], which calculates in addition what
    /// new archetype it would fit in.
    ///
    /// # Safety
    ///
    /// `archetype_id` needs to point to a valid [`Archetype`]
    pub unsafe fn get_add_archetype_id<T: Bundle>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        bundles: &mut Bundles,
        components: &mut Components,
        on_create_capacity: usize,
    ) -> ArchetypeId {
        let bundle_info = bundles.init::<T>(components);
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = bundle_info.components().clone();
        bundle_components.insert(archetype.component_ids().collect());

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
    pub unsafe fn get_sub_archetype_id<T: Bundle>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        bundles: &mut Bundles,
        components: &mut Components,
        on_create_capacity: usize,
    ) -> ArchetypeId {
        let bundle_info = bundles.init::<T>(components);
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = bundle_info.components().clone();
        bundle_components.remove(archetype.component_ids().collect());

        archetypes.get_id_from_components_or_create_with_capacity(
            components,
            &bundle_components,
            on_create_capacity,
        )
    }
}
