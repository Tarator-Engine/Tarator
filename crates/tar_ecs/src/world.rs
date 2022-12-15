use crate::{
    archetype::ArchetypePool,
    component::*,
    error::EcsError as Error,
    entity::*,
};


pub struct World {
    desc: DescriptionPool,
    arch: ArchetypePool
}

impl World {
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            desc: DescriptionPool::new(),
            arch: ArchetypePool::new()
        })
    }
    #[inline]
    pub fn entity_new(&mut self) -> Result<Entity, Error> {
        self.desc.create() 
    }
    pub fn entity_set<'a, C: ComponentTuple<'a>>(&mut self, entity: Entity, data: C) -> Result<(), Error> {
        let desc = self.desc.get_mut(entity)?;
        self.arch.set(desc, data)
    }
}

