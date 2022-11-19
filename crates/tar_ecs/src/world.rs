use crate::{ *, component::* };


type EntityId = usize;
type ComponentMask = hibitset::BitSet;


struct EntityDesc {
    id: EntityId,
    mask: ComponentMask
}


pub struct World {
    entities: Vec<EntityDesc>,
    components: Vec<ComponentPool>
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::with_capacity(MAXENTITIES),
            components: Vec::with_capacity(MAXCOMPONENTS)
        }
    }
    pub fn component_set<C: Component>(&mut self) -> Result<(), &'static str> {
        let id = C::id();
        if let None = self.components.get(id) {
            self.components.insert(id, ComponentPool::new::<C>());
            Ok(())
        } else {
            Err("Component already set!")
        }
    }
    pub fn entity_new(&mut self) -> EntityId {
        let id = self.entities.len();
        let mask = ComponentMask::new();
        let desc = EntityDesc { id, mask };
        self.entities.push(desc);

        id
    }
    pub fn entity_set<C: Component>(&mut self, entity: EntityId) -> Result<&mut C, &'static str> {
        let id = C::id();

        if let Some(pool) = self.components.get_mut(id) {
            if let Some(entity) = self.entities.get_mut(entity) {
                entity.mask.add(id as u32);
            } else {
                return Err("Entity is invalid!");
            }
            pool.get_mut::<C>(id)
        } else {
            Err("Component not set on world!")
        }
    }
    pub fn entity_unset<C: Component>(&mut self, entity: EntityId) -> Result<(), &'static str> {
        let id = C::id();

        if let Some(pool) = self.components.get_mut(id) {
            if let Some(entity) = self.entities.get_mut(entity) {
                entity.mask.remove(id as u32);
            } else {
                return Err("Entity is invalid!");
            }
            pool.clear::<C>(id)
        } else {
            Err("Component not set on world!")
        }       
    }
    pub fn entity_get<C: Component>(&mut self, entity: EntityId) -> Result<&C, &'static str> {
        let id = C::id();
        if !self.entities.get(entity).unwrap().mask.contains(id as u32) {
            return Err("Entity does not contain component anymore!");
        }
        if let Some(pool) = self.components.get(id) {
            pool.get::<C>(entity)
        } else {
            Err("Component not set on world!")
        }
    }
    pub fn entity_get_mut<C: Component>(&mut self, entity: EntityId) -> Result<&mut C, &'static str> {
        let id = C::id();
        if !self.entities.get(entity).unwrap().mask.contains(id as u32) {
            return Err("Entity does not contain component anymore!");
        }
        if let Some(pool) = self.components.get(id) {
            pool.get_mut::<C>(entity)
        } else {
            Err("Component not set on world!")
        }
    }
}

