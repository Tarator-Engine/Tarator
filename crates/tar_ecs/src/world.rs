use std::sync::atomic::{ AtomicUsize, Ordering };
use crate::{
    bundle::{ Bundles, Bundle },
    component::{ Component, Components, ComponentDescription, ComponentId, ComponentQueryMut },
    entity::{ Entities, Entity },
    archetype::{ Archetypes, ArchetypeId },
};


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct WorldId(usize);

static WORLD_COUNT: AtomicUsize = AtomicUsize::new(0);

impl WorldId {
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


#[derive(Debug)]
pub struct World {
    id: WorldId,
    archetypes: Archetypes,
    bundles: Bundles,
    components: Components,
    entities: Entities
}

impl World {
    #[inline]
    pub fn new() -> Self {
        Self {
            id: WorldId::new(),
            archetypes: Archetypes::new(),
            bundles: Bundles::new(),
            components: Components::new(),
            entities: Entities::new()
        }
    }

    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

    #[inline]
    pub fn archetypes(&self) -> &Archetypes {
        &self.archetypes
    }

    #[inline]
    pub fn bundles(&self) -> &Bundles {
        &self.bundles
    }

    #[inline]
    pub fn components(&self) -> &Components {
        &self.components
    }

    #[inline]
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    #[inline]
    pub fn component_init<T: Component>(&mut self) -> ComponentId {
        self.components.init::<T>()
    }

    #[inline]
    pub fn component_init_from_description(&mut self, description: ComponentDescription) -> ComponentId {
        self.components.init_from_description(description)
    }

    #[inline]
    pub fn component_id_from<T: Component>(&self) -> Option<&ComponentId> {
        self.components.get_id_from::<T>()
    }

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

    #[inline]
    pub fn entity_destroy(&mut self, entity: Entity) {
        let Some(entity_meta) = self.entities.destroy(entity) else {
            return;
        };

        if entity_meta.is_empty() {
            return;
        }

        // If [`Archetype`] of `entity` does change
        let archetype = self.archetypes.get_mut(entity_meta.archetype_id).expect(format!("{:#?} is invalid!", entity_meta.archetype_id).as_str());

        let replaced_entity = unsafe { archetype.drop_entity(entity_meta.index) };

        // Set the index of the moved entity
        //
        // SAFETY:
        // Entity definitely exists
        if let Some(replaced_entity) = replaced_entity {
            let replaced_entity_meta = unsafe { self.entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = entity_meta.index;
        }
    }

    pub fn entity_set<'a, T: Bundle<'a>>(&mut self, entity: Entity, data: T) {

        let Self { archetypes, bundles, components, entities, .. } = self;

        let entity_meta = entities.get_mut(entity).expect(format!("{:#?} is no more valid!", entity).as_str());

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
            // Function is unsafe in order go around the borrow checker, as well as to ignore some
            // bound checks on indices which are already checked by `self.entities` to be valid
            let archetype_id = unsafe {
                Self::get_add_archetype_id::<T>(
                    archetypes,
                    entity_meta.archetype_id,
                    bundles,
                    components,
                    1
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
        let (old_archetype, new_archetype) = archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (old_index, new_index) = (entity_meta.index, new_archetype.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        // SAFETY:
        // We initialize the remaining components right after
        let replaced_entity = unsafe { old_archetype.move_into_parent(new_archetype, old_index) };
        new_archetype.init(components, entity, data);
        
        // Set the index of the moved entity
        //
        // SAFETY:
        // Entity definitely exists
        if let Some(replaced_entity) = replaced_entity {
            let replaced_entity_meta = unsafe { entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = old_index;
        }
    }

    #[inline]
    pub fn entity_unset<'a, T: Bundle<'a>>(&mut self, entity: Entity) {

        let Self { archetypes, bundles, components, entities, .. } = self;

        let Some(entity_meta) = entities.get_mut(entity) else {
            return;
        };

        if entity_meta.is_empty() {
            return;
        }

        let archetype_id = unsafe {
            Self::get_sub_archetype_id::<T>(
                archetypes,
                entity_meta.archetype_id,
                bundles,
                components,
                1
            )
        };

        if archetype_id == entity_meta.archetype_id {
            return;
        }

        // If [`Archetype`] of `entity` does change
        let (old_archetype, new_archetype) = archetypes.get_2_mut(entity_meta.archetype_id, archetype_id);
        let (old_index, new_index) = (entity_meta.index, new_archetype.len());

        entity_meta.index = new_index;
        entity_meta.archetype_id = new_archetype.id();

        let replaced_entity = unsafe { old_archetype.move_into_child(new_archetype, old_index) };

        // Set the index of the moved entity
        //
        // SAFETY:
        // Entity definitely exists
        if let Some(replaced_entity) = replaced_entity {
            let replaced_entity_meta = unsafe { entities.get_unchecked_mut(replaced_entity.id() as usize) };
            replaced_entity_meta.index = old_index;
        }
    }

    #[inline]
    pub fn entity_get<'a, T: Bundle<'a>>(&self, entity: Entity) -> T::WrappedRef {
        let Some(meta) = self.entities.get(entity) else {
            return T::EMPTY_REF;
        };
        let Some(archetype) = self.archetypes.get(meta.archetype_id) else {
            return T::EMPTY_REF;
        };
        archetype.get::<T>(&self.components, meta.index)
    }

    #[inline]
    pub fn entity_get_mut<'a, T: Bundle<'a>>(&mut self, entity: Entity) -> T::WrappedMutRef {
        let Some(meta) = self.entities.get(entity) else {
            return T::EMPTY_MUTREF;
        };
        let Some(archetype) = self.archetypes.get_mut(meta.archetype_id) else {
            return T::EMPTY_MUTREF;
        };

        archetype.get_mut::<T>(&self.components, meta.index)
    }

    pub fn entity_query<'a, T: Bundle<'a>>(&mut self) -> Vec<Entity> {
        let bundle_info = self.bundles.init::<T>(&mut self.components);
        let archetype_id = self.archetypes.get_id_from_components_or_create_with_capacity(&self.components, bundle_info.components(), 1);
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

    #[inline]
    pub fn component_query_mut<'a, T: Bundle<'a>>(&'a mut self) -> ComponentQueryMut<'a, T> {
        let bundle_info = self.bundles.init::<T>(&mut self.components);
        let archetype_id = self.archetypes.get_id_from_components_or_create_with_capacity(
            &self.components,
            bundle_info.components(),
            1
        );

        // SAFETY:
        // Archetype was just created or gotten
        let archetype = unsafe { self.archetypes.get_unchecked(archetype_id.index()) };
        let mut archetype_ids: Vec<_> = archetype.parents().map(|id| *id).collect();
        archetype_ids.push(archetype_id);

        ComponentQueryMut::new(archetype_ids, &mut self.archetypes, &self.components)
    }


    /// SAFETY:
    /// - `archetype_id` needs to point to a valid [`Archetype`]
    pub unsafe fn get_add_archetype_id<'a, T: Bundle<'a>>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        bundles: &mut Bundles,
        components: &mut Components,
        on_create_capacity: usize
    ) -> ArchetypeId {
        let bundle_info = bundles.init::<T>(components);
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = bundle_info.components().clone();
        bundle_components.insert(archetype.component_ids().collect());

        archetypes.get_id_from_components_or_create_with_capacity(components, &bundle_components, on_create_capacity)
    }

    /// SAFETY:
    /// - `archetype_id` needs to point to a valid [`Archetype`]
    pub unsafe fn get_sub_archetype_id<'a, T: Bundle<'a>>(
        archetypes: &mut Archetypes,
        archetype_id: ArchetypeId,
        bundles: &mut Bundles,
        components: &mut Components,
        on_create_capacity: usize
    ) -> ArchetypeId {
        let bundle_info = bundles.init::<T>(components);
        let archetype = archetypes.get_unchecked(archetype_id.index());

        let mut bundle_components = bundle_info.components().clone();
        bundle_components.remove(archetype.component_ids().collect());

        archetypes.get_id_from_components_or_create_with_capacity(components, &bundle_components, on_create_capacity)
    }
}

