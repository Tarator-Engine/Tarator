mod view;

use view::EntityView;
use crate::{
    *,
    entity::{
        Entity,
        description::*,
        entity_id::*
    },
    component::{
        *,
        pool::ComponentPool,
        tuple::ComponentTuple
    }
};


/// Stores and exposes operations on [entities](Entity) and [components](Component).
///
/// Each [Entity] can have a set of components. Entities can be created, updated, removed and viewed
/// by the world.
///
/// Before components get set on entities, they have to be initialized on the world by calling
/// [`World::component_set`].
pub struct World {
    entities: DescriptionPool,
    free_entities: Vec<EntityIndex>,
    components: Vec<ComponentPool>
}

// World Implementations
impl World {
    /// Creates a new, empty world
    pub fn new() -> Self {
        Self {
            entities: DescriptionPool::new(),
            free_entities: Vec::with_capacity(MAXENTITIES),
            components: Vec::with_capacity(MAXCOMPONENTS)
        }
    }
}

// Component Implementations
impl World {
    /// Initializes a component on the world. Has to be done before any components get set on an
    /// entity. Returns [Err] if the [Component] is already set.
    pub fn component_set<C: Component>(&mut self) -> Result<(), String> {
        let cid = C::id();

        if let Some(_) = self.components.get(cid) {
            Err(format!("Component({}) already set!", cid))
        } else {
            self.components.insert(cid, ComponentPool::new::<C>());
            Ok(())
        }
    }
}

// Entity Implementations
impl World {
    /// Returns a new [Entity]
    pub fn entity_new(&mut self) -> Result<Entity, String> {
        if !self.free_entities.is_empty() {
            let Some(new_index) = self.free_entities.pop() else {
                return Err(format!("Something strange happened..."));
            }; 
            let invalid_entity = self.entities.get_mut(new_index as usize)?;
            let new_id = EntityId::new(new_index, invalid_entity.id.version());
            invalid_entity.id = new_id;

            return Ok(Entity::new(new_id));
        }
        
        let id = EntityId::new(self.entities.len() as u32, 0);
        self.entities.push(id);

        Ok(Entity::new(id))
    }


    /// Clears the [Entity]. Although [Entity] is just an u64, it should be discarded after
    /// destruction.
    pub fn entity_destroy(&mut self, entity: Entity) -> Result<(), String> {
        let id = entity.id();
        let index = id.index() as usize;

        let entity = self.entities.get_mut(index)?;

        let new_id = EntityId::versioned_invalid(id.version() + 1);
        entity.id = new_id;
        entity.mask.clear();
        self.free_entities.push(id.index());

        Ok(())
    }

    /// Sets a [Component] on an [Entity] and returns a mutable reference to said [Component]. If
    /// the [Component] is not initialized on the world or already set on the [Entity], it will return [Err].
    pub fn entity_set<C: ComponentTuple>(&mut self, entity: Entity) -> Result<(), String> {
        let eid = entity.id();
        let entity = self.entities.get_mut(eid.index() as usize)?;
        for cid in C::get_ids() {
            if !entity.mask.insert(cid) {
                return Err(format!("Component({}) already set on Entity({})!", cid, eid));
            }
        }
        
        Ok(())
    }


    /// Unsets a [Component] on an [Entity]. If the [Component] is not initialized on the world or
    /// the [Entity] has already been destroyed, it will return [Err].
    pub fn entity_unset<C: Component>(&mut self, entity: Entity) -> Result<(), String> {
        let cid = C::id();
        let eid = entity.id();
        let index = eid.index() as usize;

        let Some(pool) = self.components.get(cid) else {
            return Err(format!("Component({}) not set on world!", cid));
        };

        let entity = self.entities.get_mut(index)?;

        // if versions in EntityId differ, do not override newer entity
        if entity.id != eid {
            // Could also return Ok(())
            return Err(format!("Entity({}) already destroyed!", eid));
        }

        entity.mask.remove(&cid);
        pool.clear::<C>(index)?;

        Ok(())
    }


    // TODO take tuple with components as generic
    pub fn entity_operate<C: Component>(&mut self, entity: Entity, func: impl FnOnce(&mut C)) -> Result<(), String> {
        let cid = C::id();
        let eid = entity.id();
        let index = eid.index() as usize;

        let Some(pool) = self.components.get(cid) else {
            return Err(format!("Component({}) not set on world!", cid));
        };

        let entity = self.entities.get_mut(eid.index() as usize)?;

        if !entity.mask.contains(&cid) {
            return Err(format!("Entity({}) does not contain Component({})", eid, cid));
        }

        let component = pool.get_mut::<C>(index)?;
        Ok(func(component))
    }


    pub fn entity_view<C: ComponentTuple>(&mut self) -> Result<EntityView, String> {
        
        let mut mask = ComponentMask::new(); 
        for cid in C::get_ids() {
            if !mask.insert(cid) {
                return Err(format!("Component({}) already viewed!", cid));
            }
        }

        Ok(EntityView::new(self, mask))
    }
}

