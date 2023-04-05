pub mod archetype;
pub mod bundle;
pub mod callback;
pub mod component;
pub mod entity;
pub mod store;
pub mod type_info;
pub mod world;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub type World = super::world::World<super::type_info::Local>;

    pub use super::bundle::{Bundle, CloneBundle};
    pub use super::callback::{Callback, CallbackName, InnerCallback};
    pub use super::component::{Component, ComponentInfo, ComponentName, Empty};
    pub use super::entity::Entity;
    pub use super::store::table::Indexer;
    pub use tar_ecs_macros::{Callback, Component};
}
