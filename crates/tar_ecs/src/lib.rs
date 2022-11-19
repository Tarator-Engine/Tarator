mod world;
mod component;
mod type_pool;

pub mod prelude {
    pub use super::world::World;
    pub use super::component::{ ComponentId, Component };
    pub use macros::Component;
}


const MAXENTITIES: usize = 64;
const MAXCOMPONENTS: usize = 16;

