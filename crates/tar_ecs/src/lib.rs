pub mod archetype;
pub mod bundle;
pub mod callback;
pub mod component;
pub mod entity;
pub mod query;
pub mod store;
pub mod type_info;
pub mod world;

#[cfg(test)]
mod tests;

extern crate self as tar_ecs;

pub use tar_ecs_macros as macros;

pub mod prelude {
    pub type World = super::world::World<super::type_info::Local>;
    pub type Query<'a, T> = super::query::Query<'a, T, super::type_info::Local>;
    pub type QueryMut<'a, T> = super::query::Query<'a, T, super::type_info::Local>;

    pub use super::bundle::{Bundle, CloneBundle, UBundleId};
    pub use super::callback::{Callback, InnerCallback, UCallbackId};
    pub use super::component::{Component, ComponentInfo, UComponentId};
    pub use super::entity::Entity;
    pub use super::store::table::Indexer;
}
