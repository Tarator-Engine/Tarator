use std::{
    sync::atomic::{ AtomicUsize, Ordering },
    any::TypeId
};

use crate::{
    bundle::{ Bundles, Bundle },
    component::{ Component, Components, ComponentDescription, ComponentId },
    entity::{ Entities, Entity },
    archetype::Archetypes
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
        self.entities.create()
    }

    #[inline]
    pub fn entity_destroy(&mut self, entity: Entity) {
        let meta = self.entities.destroy(entity);
        todo!("Still got to delete the components with {:#?}", meta);
    }

    #[inline]
    pub fn entity_set<T: Bundle>(&mut self, entity: Entity, data: T) {
        let entity_meta = self.entities.get_mut(entity).expect("Entity was invalid!");
        let bundle_info = self.bundles.init::<T>(&mut self.components);
        let archetype = match self.archetypes.get_from_bundle_mut(bundle_info.id()) {
            Some(archetype) => archetype,
            None => {
                let id = self.archetypes.create_with_capacity(bundle_info, &self.components, 1);

                // SAFETY:
                // Archetype was just created
                unsafe { self.archetypes.get_unchecked_mut(id.index()) }
            }
        };
        entity_meta.index = archetype.len();
        entity_meta.archetype_id = archetype.id();
        archetype.set(&self.components, entity, data);
    }

    #[inline]
    pub fn entity_unset<T: Bundle>(&mut self, entity: Entity) -> T {
        let meta = self.entities.get_mut(entity);
        let info = self.bundles.init::<T>(&mut self.components);
        todo!("Still gotta unset using {:#?} and {:#?}", meta, info);
    }

    #[inline]
    pub fn entity_get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let meta = self.entities.get(entity)?;
        let archetype = self.archetypes.get(meta.archetype_id)?;
        archetype.get(&self.components, meta.index)
    }

    #[inline]
    pub fn entity_get_mut<T: Component>(&mut self, entity: Entity) -> &mut T {
        let meta = self.entities.get(entity);
        let info = self.components.get_id_from::<T>();
        todo!("Still gotta get using {:#?} and {:#?}", meta, info);
    }
}

