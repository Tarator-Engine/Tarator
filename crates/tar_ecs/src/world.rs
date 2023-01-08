use std::sync::{ Arc, Mutex };
use crate::{
    component::{
        Component,
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
    world: Arc<Mutex<InnerWorld>>
}

impl World {
    #[inline]
    pub fn new() -> Self {
        Self { world: Arc::new(Mutex::new(InnerWorld::new())) }
    } 
    #[inline]
    pub fn entity_new(&mut self) -> Result<Entity, Error> {
        let Ok(mut world) = self.world.lock() else {
            return Err(Error::MutexError);
        };
        world.entity_new()
    }
    #[inline]
    pub fn entity_set<C: ComponentTuple>(&mut self, entity: Entity, data: C) -> Result<(), Error> {
        let Ok(mut world) = self.world.lock() else {
            return Err(Error::MutexError);
        };
        world.entity_set(entity, data)

    }
    #[inline]
    pub fn entity_get<C: Component>(&self, entity: Entity) -> Result<StorePtr<C>, Error> {
        let Ok(world) = self.world.lock() else {
            return Err(Error::MutexError);
        };
        world.entity_get(entity)
    }
    pub fn entity_view<C: ComponentTuple>(&self) -> Result<EntityView, Error> {
        todo!()
    }
    pub fn component_view<C: ComponentTuple>(&self) -> Result<ComponentView, Error> {
        todo!()
    }
}


pub struct InnerWorld {
    pub(crate) desc: DescPool,
    pub(crate) arche: ArchePool
}

impl InnerWorld {
    #[inline]
    fn new() -> Self {
        Self {
            desc: DescPool::new(),
            arche: ArchePool::new()
        }
    }
    #[inline]
    fn entity_new(&mut self) -> Result<Entity, Error> {
        self.desc.create() 
    }
    #[inline]
    fn entity_set<C: ComponentTuple>(&mut self, entity: Entity, data: C) -> Result<(), Error> {
        let desc = self.desc.get_mut(entity.id)?;
        self.arche.set(desc, data)
    }
    #[inline]
    fn entity_get<C: Component>(&self, entity: Entity) -> Result<StorePtr<C>, Error> {
        let desc = self.desc.get(entity.id)?;
        self.arche.get(desc)
    }
    fn entity_view<C: ComponentTuple>(&self) -> Result<EntityView, Error> {
        todo!()
    }
    fn component_view<C: ComponentTuple>(&self) -> Result<ComponentView, Error> {
        todo!()
    }
}

