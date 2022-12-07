pub mod entity_id;
pub mod description;

use entity_id::*;


#[derive(Clone, Copy, Debug)]
pub struct Entity {
    id: EntityId
}

impl Entity {
    #[inline]
    pub(crate) fn new(id: EntityId) -> Self {
        Self { id }
    }
    #[inline]
    pub(crate) fn id(&self) -> EntityId {
        self.id
    }
}

