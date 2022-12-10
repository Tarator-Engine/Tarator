pub(crate) mod pool;
pub mod tuple;


pub(crate) type ComponentMask = std::collections::HashSet<ComponentId>;

pub type ComponentId = usize;

/// implement by using #[derive(Component)]
pub trait Component: Send + Sync {
    fn id() -> ComponentId;
}

