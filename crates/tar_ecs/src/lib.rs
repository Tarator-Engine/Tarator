mod component;
mod entity;
mod type_pool;
mod world;

pub mod prelude {
    pub use super::entity::Entity;
    pub use super::world::World;
    pub use super::component::{ ComponentId, Component };
    pub use macros::Component;
}


const MAXENTITIES: usize = 64;
const MAXCOMPONENTS: usize = 16;

