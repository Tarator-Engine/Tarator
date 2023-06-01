use std::cell::UnsafeCell;

use crate::{
    bundle::BundleId,
    store::{sparse::MutSparseSet, table::Table},
    type_info::TypeInfo
};

#[derive(Clone, Debug)]
pub struct Archetype {
    table: Table,
    parents: Vec<BundleId>,
}

impl Archetype {
    #[inline]
    pub fn new(bundle_id: BundleId, parents: Vec<BundleId>, type_info: &impl TypeInfo) -> Self {
        Self {
            table: Table::new(bundle_id, type_info),
            parents,
        }
    }

    #[inline]
    pub fn table(&self) -> &Table {
        &self.table
    }

    #[inline]
    pub fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    pub fn parents(&self) -> &Vec<BundleId> {
        &self.parents
    }
}

#[derive(Debug)]
pub struct Archetypes {
    archetypes: UnsafeCell<MutSparseSet<BundleId, Archetype>>,
}

impl Clone for Archetypes {
    fn clone(&self) -> Self {
        Self {
            archetypes: UnsafeCell::new(unsafe { (*self.archetypes.get()).clone() })
        }
    }
}

impl Default for Archetypes {
    #[inline]
    fn default() -> Self {
         Self {
            archetypes: UnsafeCell::new(MutSparseSet::new()),
        }
    }    
}

impl Archetypes {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn try_init(&self, bundle_id: BundleId, type_info: &impl TypeInfo) {
        if self.get(bundle_id).is_some() {
            return;
        }

        let parents = type_info
            .get_bundle_info(bundle_id, |info| {
                let mut parents = Vec::new();

                unsafe {
                    for (id, archetype) in (*self.archetypes.get()).iter_mut() {
                        type_info.get_bundle_info(*id, |parent_info| {
                            if info.is_superset(parent_info) {
                                archetype.parents.push(bundle_id);
                            } else if info.is_subset(parent_info) {
                                parents.push(*id)
                            }
                        });
                    }
                }

                parents
            })
            .expect("Bundle wasn't initialized!");

        let archetype = Archetype::new(bundle_id, parents, type_info);
        unsafe { (*self.archetypes.get()).insert(bundle_id, archetype) };
    }
}

impl Archetypes {
    #[inline]
    pub fn get(&self, bundle_id: BundleId) -> Option<&Archetype> {
        unsafe { (*self.archetypes.get()).get(bundle_id) }
    }

    #[inline]
    pub fn get_mut(&mut self, bundle_id: BundleId) -> Option<&mut Archetype> {
        unsafe { (*self.archetypes.get()).get_mut(bundle_id) }
    }

    #[inline]
    pub fn get_2_mut(
        &mut self,
        i1: BundleId,
        i2: BundleId,
    ) -> Option<(&mut Archetype, &mut Archetype)> {
        unsafe { (*self.archetypes.get()).get_2_mut(i1, i2) }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&BundleId, &Archetype)> {
        unsafe { (*self.archetypes.get()).iter() }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&BundleId, &mut Archetype)> {
        unsafe { (*self.archetypes.get()).iter_mut() }
    }

}
