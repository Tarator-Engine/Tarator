use std::alloc::{ Layout, LayoutError };

use crate::{
    entity::EntityId,
    id::{Index, Id}, component::ComponentSet
};

#[derive(Debug)]
pub enum EcsError {
    InvalidEntity(EntityId),
    InvalidIndex(Index),
    ClearedEntity(EntityId),
    UnsetComponent(ComponentSet),

    // Memory Errors
    Layout(Result<Layout, LayoutError>),
    DataMoved(Id),

    MutexError
}
