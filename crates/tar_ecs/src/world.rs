use crate::{
    component::{
        arche::ArchePool,
        tuple::ComponentTuple,
        view::ComponentView,
        store::StorePtr
    },
    entity::{
        Entity,
        desc::DescPool,
        view::EntityView
    },
    error::EcsError as Error
};


pub struct World {
    desc: DescPool,
    arch: ArchePool
}

impl World {
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            desc: DescPool::new(),
            arch: ArchePool::new()
        })
    }
    #[inline]
    pub fn entity_new(&mut self) -> Result<Entity, Error> {
        self.desc.create() 
    }
    pub fn entity_set<C: ComponentTuple>(&mut self, entity: Entity, data: C) -> Result<(), Error> {
        let desc = self.desc.get_mut(entity)?;
        self.arch.set(desc, data)
    }
    pub fn entity_get<C: ComponentTuple>(&self, entity: Entity) -> Result<StorePtr<C>, Error> {
        let desc = self.desc.get(entity)?;
        self.arch.get(desc)
    }
    pub fn entity_view<C: ComponentTuple>(&self) -> Result<EntityView, Error> {
        todo!()
    }
    pub fn component_view<C: ComponentTuple>(&self) -> Result<ComponentView, Error> {
        todo!()
    }
}

