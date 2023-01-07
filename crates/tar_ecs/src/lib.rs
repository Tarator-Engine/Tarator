mod component;
mod entity;
mod error;
mod id;
#[cfg(test)]
mod tests;
mod world;


pub mod prelude {
    pub use super::{
        entity::Entity,
        error::EcsError,
        component::{
            Component,
            ComponentId
        },
        id::Id,
        world::*
    };
    pub use macros::Component;
}

