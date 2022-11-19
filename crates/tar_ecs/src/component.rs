use crate::{ MAXENTITIES, type_pool::TypePool };


pub type ComponentId = usize;

pub trait Component: Send + Sync {
    fn id() -> ComponentId;
}


pub struct ComponentPool {
    pool: TypePool
}

impl ComponentPool {
    pub fn new<C: Component>() -> Self {
        Self {
            pool: unsafe{TypePool::new::<C>(MAXENTITIES)}
        }
    }
    pub fn get<C: Component>(&self, index: usize) -> Result<&C, &'static str> {
        if index > self.pool.len() { return Err("Out of bounds!"); }
        Ok(unsafe{self.pool.get::<C>(index)})
    }
    pub fn get_mut<C: Component>(&self, index: usize) -> Result<&mut C, &'static str> {
        if index > self.pool.len() { return Err("Out of bounds!"); }
        Ok(unsafe{self.pool.get_mut::<C>(index)})
    }
    pub fn clear<C: Component>(&self, index: usize) -> Result<(), &'static str> {
        if index > self.pool.len() { return Err("Out of bounds!"); }
        Ok(unsafe{self.pool.clear::<C>(index)})
    }
    pub fn as_slice<C: Component>(&self) -> &[C] {
        unsafe{self.pool.as_slice::<C>()}
    }
    pub fn as_slice_mut<C: Component>(&self) -> &mut [C] {
        unsafe{self.pool.as_slice_mut::<C>()}
    }
}
