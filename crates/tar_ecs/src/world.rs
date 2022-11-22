use crate::{ *, entity::*, component::* };


type ComponentMask = hibitset::BitSet;


struct EntityDesc {
    id: EntityId,
    mask: ComponentMask
}


pub struct World {
    entities: Vec<EntityDesc>,
    free_entities: Vec<EntityIndex>,
    components: Vec<ComponentPool>
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::with_capacity(MAXENTITIES),
            free_entities: Vec::with_capacity(MAXENTITIES),
            components: Vec::with_capacity(MAXCOMPONENTS)
        }
    }

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

    pub fn entity_new(&mut self) -> Result<Entity, String> {
        if !self.free_entities.is_empty() {
            let Some(new_index) = self.free_entities.pop() else {
                return Err(format!("Something strange happened..."));
            }; 
            let Some(invalid_entity) = self.entities.get_mut(new_index as usize) else {
                return Err(format!("Something strange happened with Entity(index: {})...", new_index));
            };
            let new_id = EntityId::new(new_index, invalid_entity.id.version());
            invalid_entity.id = new_id;

            return Ok(Entity::new(new_id));
        }
        
        let id = EntityId::new(self.entities.len() as u32, 0);
        let desc = EntityDesc {
            id,
            mask: ComponentMask::new()
        };
        self.entities.push(desc);

        Ok(Entity::new(id))
    }

    pub fn entity_destroy(&mut self, entity: Entity) -> Result<(), String> {
        let id = entity.id();
        let index = id.index() as usize;

        let Some(entity) = self.entities.get_mut(index) else {
            return Err(format!("Entity({}) is invalid!", id));
        };

        let new_id = EntityId::versioned_invalid(id.version() + 1);
        entity.id = new_id;
        entity.mask.clear();
        self.free_entities.push(id.index());

        Ok(())
    }

    pub fn entity_set<C: Component>(&mut self, entity: Entity) -> Result<&mut C, String> {
        let cid = C::id();
        let eid = entity.id();

        let Some(pool) = self.components.get(cid) else {
            return Err(format!("Component({}) not set on world!", cid));
        };

        let Some(entity) = self.entities.get_mut(eid.index() as usize) else {
            return Err(format!("Entity({}) is invalid!", eid));
        };

        entity.mask.add(cid as u32);
        pool.get_mut::<C>(eid.index() as usize)
    }

    pub fn entity_unset<C: Component>(&mut self, entity: Entity) -> Result<(), String> {
        let cid = C::id();
        let eid = entity.id();
        let index = eid.index() as usize;

        let Some(pool) = self.components.get(cid) else {
            return Err(format!("Component({}) not set on world!", cid));
        };

        let Some(entity) = self.entities.get_mut(index) else {
            return Err(format!("Entity({}) is invalid!", eid));
        };

        // if versions in EntityId differ, do not override newer entity
        if entity.id != eid {
            // Could also return Ok(())
            return Err(format!("Entity({}) already deleted!", eid));
        }

        entity.mask.remove(cid as u32);
        pool.clear::<C>(index)?;

        Ok(())
    }

    pub fn entity_operate<C: Component>(&mut self, entity: Entity, func: impl FnOnce(&mut C)) -> Result<(), String> {
        let cid = C::id();
        let eid = entity.id();
        let index = eid.index() as usize;

        let Some(pool) = self.components.get(cid) else {
            return Err(format!("Component({}) not set on world!", cid));
        };

        let Some(entity) = self.entities.get_mut(eid.index() as usize) else {
            return Err(format!("Entity({}) is invalid!", eid));
        };

        if !entity.mask.contains(cid as u32) {
            return Err(format!("Entity({}) does not contain Component({})", eid, cid));
        }

        let component = pool.get_mut::<C>(index)?;
        Ok(func(component))
    }
}

