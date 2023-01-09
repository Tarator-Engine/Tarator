pub(crate) mod desc;
pub(crate) mod view;


use crate::id::Id;

pub(crate) type EntityId = Id;

#[derive(Clone, Copy)]
pub struct Entity {
    pub(crate) id: EntityId
}

