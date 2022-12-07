use crate::{ MAXENTITIES, type_pool::TypePool };


pub type ComponentId = usize;

/// implement by using #[derive(Component)]
pub trait Component: Send + Sync {
    fn id() -> ComponentId;
}


/// Safe wrapper around TypePool. Stores one type of components.
/// TODO Track cleared components and sort data
pub struct ComponentPool {
    pool: TypePool
}

impl ComponentPool {
    pub fn new<C: Component>() -> Self {
        Self {
            pool: unsafe{TypePool::new::<C>(MAXENTITIES)}
        }
    }
    pub fn get<C: Component>(&self, index: usize) -> Result<&C, String> {
        if index > self.pool.len() { return Err(format!("index({}) is out of bounds!", index)); }
        Ok(unsafe{self.pool.get::<C>(index)})
    }
    pub fn get_mut<C: Component>(&self, index: usize) -> Result<&mut C, String> {
        if index > self.pool.len() { return Err(format!("index({}) is out of bounds!", index)); }
        Ok(unsafe{self.pool.get_mut::<C>(index)})
    }
    pub fn clear<C: Component>(&self, index: usize) -> Result<(), String> {
        if index > self.pool.len() { return Err(format!(" index({}) id out of bounds!", index)); }
        Ok(unsafe{self.pool.clear::<C>(index)})
    }
    pub fn as_slice<C: Component>(&self) -> &[C] {
        unsafe{self.pool.as_slice::<C>()}
    }
    pub fn as_slice_mut<C: Component>(&self) -> &mut [C] {
        unsafe{self.pool.as_slice_mut::<C>()}
    }
}

