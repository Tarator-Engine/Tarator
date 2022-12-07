use crate::{
    *,
    entity::{
        Entity,
        description::*,
        entity_id::*
    },
    component::*,
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

impl World {

    /// Creates a new, empty world
    pub fn new() -> Self {
        Self {
            entities: DescriptionPool::new(),
            free_entities: Vec::with_capacity(MAXENTITIES),
            components: Vec::with_capacity(MAXCOMPONENTS)
        }
    }

    /// Initializes a component on the world. Has to be done before any components get set on an
    /// entity. Returns [Err] if the [Component] is already set.
    pub fn component_set<C: Component>(&mut self) -> Result<(), String> {
        let cid = C::id();

        match self.components.get(cid) {
            Some(_) => Err(format!("Component({}) already set!", cid)),
            None => {
                self.components.insert(cid, ComponentPool::new::<C>());
                Ok(())
            }
        }
    }

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
    pub fn entity_set<C: Component>(&mut self, entity: Entity) -> Result<&mut C, String> {
        let cid = C::id();
        let eid = entity.id();

        let Some(pool) = self.components.get(cid) else {
            return Err(format!("Component({}) not set on world!", cid));
        };

        let entity = self.entities.get_mut(eid.index() as usize)?;

        if !entity.mask.insert(cid) {
            return Err(format!("Component({}) already set on Entity({})!", cid, eid));
        }
        
        pool.get_mut::<C>(eid.index() as usize)
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

    pub fn view<'a>(&'a mut self) -> WorldViewBuilder<'a> {
        WorldViewBuilder::new(self)
    }
}



pub struct WorldViewBuilder<'a> {
    cids: Vec<ComponentId>,
    world: &'a mut World
}

impl<'a> WorldViewBuilder<'a> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self {
            cids: Vec::new(),
            world
        }
    }
    pub fn wish<C: Component>(mut self) -> Self {
        self.cids.push(C::id());
        self
    }
    pub fn get(self) -> Result<WorldView, String> {
        WorldView::new(self.world, self.cids)
    }
}


pub struct WorldView {
    entities: Vec<Desc>,
    mask: ComponentMask,
    index: EntityIndex
}

impl WorldView {
    pub(crate) fn new(world: &mut World, cids: Vec<ComponentId>) -> Result<WorldView, String> {

        let mut mask = ComponentMask::new();
        for cid in cids {
            if !mask.insert(cid) {
                return Err(format!("Can not create WorldView with multiple Component({})", cid));
            }
        }

        Ok(Self {
            entities: world.entities.get_vec_clone(),
            mask,
            index: 0
        })
    }

    fn is_index_valid(&self) -> bool {
        let Some(entity) = self.entities.get(self.index as usize) else {
            return false;
        };

        //     Is EntityId valid      and    all components    or  masks match
        EntityId::is_valid(entity.id) && self.mask.is_empty() || self.mask == entity.mask
    }
}

impl Iterator for WorldView {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entities.len() as u32 {
            return None;
        }

        // while entities do not match the view, check next
        while !self.is_index_valid() && self.index < self.entities.len() as u32 {
            self.index += 1; 
        }

        let Some(entity) = self.entities.get(self.index as usize) else {
            return None;
        };
        self.index += 1; 

        Some(Entity::new(entity.id))
    }
}

