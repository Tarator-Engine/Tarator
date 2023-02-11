pub mod archetype;
pub mod bundle;
pub mod component;
pub mod entity;
pub mod store;
pub mod world;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use tar_ecs_macros::Component;
    pub use super::component::Component;
    pub use super::entity::Entity;
    pub use super::world::World;
}

