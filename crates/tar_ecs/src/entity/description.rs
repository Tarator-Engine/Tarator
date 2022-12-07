use super::entity_id::*;
use crate::prelude::ComponentId;

pub(crate) type ComponentMask = std::collections::HashSet<ComponentId>;

#[derive(Clone)]
pub(crate) struct Desc {
    pub(crate) id: EntityId,
    pub(crate) mask: ComponentMask
}

pub(crate) struct DescriptionPool {
    descs: Vec<Desc>
}

impl DescriptionPool {
    pub(crate) fn new() -> Self {
        Self {
            descs: Vec::new()
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.descs.len()
    }

    pub(crate) fn push(&mut self, id: EntityId) {
        let desc = Desc {
            id, mask: ComponentMask::new()
        };
        self.descs.push(desc)
    }

    pub(crate) fn get(&self, index: usize) -> Result<&Desc, String> {
        let Some(desc) = self.descs.get(index) else {
            return Err(format!("Entity({}) is invalid!", index));
        };
        Ok(desc)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Result<&mut Desc, String> {
        let Some(desc) = self.descs.get_mut(index) else {
            return Err(format!("Entity({}) is invalid!", index));
        };
        Ok(desc)
    }

    pub(crate) fn get_vec_clone(&self) -> Vec<Desc> {
        self.descs.clone()
    }
}

