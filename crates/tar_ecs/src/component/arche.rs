use std::{sync::{Arc, atomic::AtomicBool}, mem::size_of};

use super::{
    Component, ComponentSet,
    tuple::*,
    store::*
};
use crate::{
    error::EcsError as Error,
    entity::desc::Desc, id::IdTrait
};


pub(crate) type ArcheId = usize;

pub(crate) struct Arche {
    id: ArcheId,
    set: ComponentSet,
    parents: Vec<ArcheId>,
    pub(crate) units: Vec<TupleUnit>,
    pub(crate) store: Arc<Store>
}

impl Arche {
    fn new(id: ArcheId, set: ComponentSet, units: Vec<TupleUnit>) -> Self {
        let tmp = units[units.len() - 1];
        let size = tmp.offset + tmp.size() + size_of::<AtomicBool>();
        Self { parents: Vec::new(), id, set, units, store: Arc::new(Store::new(size)) }
    }
    fn set(&self, desc: &mut Desc, data: DataUnit) -> Result<(), Error> {
        let offset = 'getter: {
            for unit in &self.units {
                if unit.id == data.id {
                    break 'getter unit.offset;
                }
            }
            return Err(Error::InvalidIndex(data.id));
        };

        if !desc.is_index_valid() {
            desc.index = self.store.create()?;
        }
        self.store.set(desc.index, offset, data)?;

        Ok(())
    }
}


pub(crate) struct ArchePool {
    pub(crate) arche: Vec<Arche>
}

impl ArchePool {
    pub(crate) fn new() -> Self {
        Self {
            arche: Vec::new() 
        }
    }
    // TODO: Relocation of Component Data
    pub(crate) fn set<C: ComponentTuple>(&mut self, desc: &mut Desc, data: C) -> Result<(), Error> {
        let arche = 'getter: {
            let set = C::set();

            if desc.id.is_index_valid() {
                let Some(arche) = self.arche.get(desc.id) else {
                    return Err(Error::InvalidIndex(desc.id));
                };
                break 'getter arche;
            }

            let new_id = self.arche.len();
            let mut new_arche = Arche::new(new_id, set, C::tuple_units().collect());
            for arche in &mut self.arche {
                if arche.set.is_superset(&new_arche.set) {
                    new_arche.parents.push(arche.id);
                } else if arche.set.is_subset(&new_arche.set) {
                    arche.parents.push(new_arche.id);
                }
            }

            self.arche.push(new_arche);
            // no need for bound checking here
            unsafe { self.arche.get_unchecked_mut(new_id) }
        };

        for unit in data.data_units() {
            arche.set(desc, unit)?;
        }

        Ok(())
    }
    pub(crate) fn get<C: Component>(&self, desc: &Desc) -> Result<StorePtr<C>, Error> {
        todo!()
    }
}

