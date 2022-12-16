use crate::{
    error::EcsError as Error,
    id::*
};

pub(crate) type EntityId = Id;
// index = ArchetypeId
pub(crate) type DescriptionId = Id;

pub struct Entity {
    id: EntityId
}


pub(crate) struct Description {
    pub(crate) id: DescriptionId,
    pub(crate) index: usize
}

impl Description {
    #[inline]
    pub(crate) fn is_index_valid(&self) -> bool {
        self.index != usize::MAX
    }
}


pub(crate) struct DescriptionPool {
    free: Vec<EntityId>,
    desc: Vec<Description>
}

impl DescriptionPool {
    pub(crate) fn new() -> Self {
        Self {
            free: Vec::new(),
            desc: Vec::new()
        } 
    }
    pub(crate) fn create(&mut self) -> Result<Entity, Error> {
        let Some(unfreed) = self.free.pop() else {
            let index = self.desc.len();
            let desc = Description {
                id: DescriptionId::versioned_invalid(0),
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
        *desc = Description {
            id: DescriptionId::versioned_invalid(version),
            index: usize::MAX
        };
        Ok(Entity { id: EntityId::new(index, version) })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity) -> Result<&mut Description, Error> {
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

