use crate::{
    entity::EntityId,
    id::Index
};

#[derive(Debug)]
pub enum EcsError {
    InvalidEntity(EntityId),
    InvalidIndex(Index),
    ClearedEntity(EntityId)
}
