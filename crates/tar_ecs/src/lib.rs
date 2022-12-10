mod component;
mod entity;
mod world;

pub mod prelude {
    pub use super::entity::Entity;
    pub use super::world::World;
    pub use super::component::{ Component, ComponentId };
    pub use macros::Component;
}


const MAXENTITIES: usize = 256;
const MAXCOMPONENTS: usize = 16;

