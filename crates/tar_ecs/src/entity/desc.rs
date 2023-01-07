use super::*;
use crate::{
    error::EcsError as Error,
    id::*
};

// index = ArchetypeId
pub(crate) type DescId = Id;

pub(crate) struct Desc {
    pub(crate) id: DescId,
    pub(crate) index: usize
}

impl Desc {
    #[inline]
    pub(crate) fn is_index_valid(&self) -> bool {
        self.index != usize::MAX
    }
}


pub(crate) struct DescPool {
    free: Vec<EntityId>,
    desc: Vec<Desc>
}

impl DescPool {
    pub(crate) fn new() -> Self {
        Self {
            free: Vec::new(),
            desc: Vec::new()
        } 
    }

    pub(crate) fn create(&mut self) -> Result<Entity, Error> {
        let Some(unfreed) = self.free.pop() else {
            let index = self.desc.len();
            let desc = Desc {
                id: DescId::versioned_invalid(0),
                index: usize::MAX
            };
            self.desc.push(desc);
            return Ok(Entity { id: EntityId::new(index, 0) })
        };

        let version = unfreed.get_version();
        let index = unfreed.get_index();
        let Some(desc) = self.desc.get_mut(index) else {
            return Err(Error::InvalidIndex(index));
        };
        *desc = Desc {
            id: DescId::versioned_invalid(version),
            index: usize::MAX
        };
        Ok(Entity { id: EntityId::new(index, version) })
    }

    pub(crate) fn destroy(&mut self, entity: Entity) -> Result<(), Error> {
        let desc = self.get_mut(entity)?;
        *desc = Desc {
            id: DescId::versioned_invalid(desc.id.get_version() + 1),
            index: usize::MAX
        };
        self.free.push(entity.id);
        Ok(())
    }

    pub(crate) fn get(&self, entity: Entity) -> Result<&Desc, Error> {
        let id = entity.id;
        let Some(desc) = self.desc.get(id.get_index()) else {
            return Err(Error::InvalidEntity(id));
        };
        if desc.id.get_version() != id.get_version() {
            return Err(Error::ClearedEntity(id));
        }
        Ok(desc)       
    }

    pub(crate) fn get_mut(&mut self, entity: Entity) -> Result<&mut Desc, Error> {
        let id = entity.id;
        let Some(desc) = self.desc.get_mut(id.get_index()) else {
            return Err(Error::InvalidEntity(id));
        };
        if desc.id.get_version() != id.get_version() {
            return Err(Error::ClearedEntity(id));
        }
        Ok(desc)
    }
}

