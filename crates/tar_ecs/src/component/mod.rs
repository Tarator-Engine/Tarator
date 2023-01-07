pub(crate) mod arche;
pub(crate) mod store;
pub mod tuple;
pub mod view;


use std::collections::HashSet;

pub type ComponentId = usize;
pub type ComponentSet = HashSet<ComponentId>;

/// implement by using #[derive(Component)]
pub trait Component {
    fn id() -> ComponentId;
}

