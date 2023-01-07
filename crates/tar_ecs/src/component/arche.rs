use super::{
    ComponentSet,
    tuple::{ComponentTuple, DataUnit},
    store::*
};
use crate::{
    error::EcsError as Error,
    entity::desc::Desc
};


pub(crate) type ArcheId = usize;

struct Arche {
    id: ArcheId,
    set: ComponentSet,
    parents: Vec<ArcheId>,
    store: Store
}

impl Arche {
    fn new(id: ArcheId, set: ComponentSet) -> Self {
        Self { parents: Vec::new(), id, set, store: Store::new() }
    }
    fn set(&self, desc: &mut Desc, unit: DataUnit) -> Result<(), Error> {
        todo!() 
    }
}


pub(crate) struct ArchePool {
    arche: Vec<Arche>
}

impl ArchePool {
    pub(crate) fn new() -> Self {
        Self {
            arche: Vec::new() 
        }
    }
    pub(crate) fn set<C: ComponentTuple>(&mut self, desc: &mut Desc, data: C) -> Result<(), Error> {
        let arche = 'getter: {
            let set = C::set();

            for arche in &mut self.arche {
                if arche.set == set {
                    break 'getter arche;
                }
            }

            let new_id = self.arche.len();
            let mut new_arche = Arche::new(new_id, set);
            for arche in &mut self.arche {
                if arche.set.is_superset(&new_arche.set) {
                    new_arche.parents.push(arche.id);
                } else if arche.set.is_subset(&new_arche.set) {
                    arche.parents.push(new_arche.id);
                }
            }

            self.arche.push(new_arche);
            self.arche.get_mut(new_id).unwrap()
        };
        for unit in data.data_units() {
            arche.set(desc, unit)?;
        }
        
        todo!()
    }
    pub(crate) fn get<C: ComponentTuple>(&self, desc: &Desc) -> Result<StorePtr<C>, Error> {
        todo!()
    }
}

