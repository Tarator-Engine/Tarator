use super::*;
use crate::{
    entity::{ *, description::Desc },
    component::*
};


pub struct EntityView {
    entities: Vec<Desc>,
    mask: ComponentMask,
    index: EntityIndex
}

impl EntityView {
    pub(crate) fn new(world: &mut World, mask: ComponentMask) -> Self {

        Self {
            entities: world.entities.get_vec_clone(),
            mask,
            index: 0
        }
    }

    fn is_index_valid(&self) -> bool {
        let Some(entity) = self.entities.get(self.index as usize) else {
            return false;
        };

        EntityId::is_valid(entity.id) && entity.mask.is_superset(&self.mask)
    }
}

impl Iterator for EntityView {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entities.len() as u32 {
            return None;
        }

        // while entities do not match the view, check next
        while !self.is_index_valid() && self.index < self.entities.len() as u32 {
            self.index += 1; 
        }

        let Some(entity) = self.entities.get(self.index as usize) else {
            return None;
        };
        self.index += 1; 

        Some(Entity::new(entity.id))
    }
}

