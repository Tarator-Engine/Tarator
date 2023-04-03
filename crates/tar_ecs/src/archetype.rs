use crate::{
    bundle::BundleId,
    store::{sparse::MutSparseSet, table::Table},
    type_info::TypeInfo,
};

#[derive(Debug)]
pub struct Archetype {
    table: Table,
    parents: Vec<BundleId>,
}

impl Archetype {
    #[inline]
    pub fn new(bundle_id: BundleId, parents: Vec<BundleId>, type_info: &impl TypeInfo) -> Self {
        Self {
            table: unsafe { Table::new(bundle_id, type_info) },
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
    archetypes: MutSparseSet<BundleId, Archetype>,
}

impl Archetypes {
    #[inline]
    pub unsafe fn new() -> Self {
        Self {
            archetypes: MutSparseSet::new(),
        }
    }

    #[inline]
    pub fn try_init(&mut self, bundle_id: BundleId, type_info: &impl TypeInfo) {
        if self.get(bundle_id).is_some() {
            return;
        }

        let parents = type_info
            .get_bundle_info(bundle_id, |info| {
                let mut parents = Vec::new();

                for (id, archetype) in self.archetypes.iter_mut() {
                    type_info.get_bundle_info(*id, |parent_info| {
                        if info.is_superset(parent_info) {
                            archetype.parents.push(bundle_id);
                        } else if info.is_subset(parent_info) {
                            parents.push(*id)
                        }
                    });
                }

                parents
            })
            .expect("Bundle wasn't initialized!");

        let archetype = Archetype::new(bundle_id, parents, type_info);
        self.archetypes.insert(bundle_id, archetype);
    }
}

impl Archetypes {
    #[inline]
    pub fn get(&self, bundle_id: BundleId) -> Option<&Archetype> {
        self.archetypes.get(bundle_id)
    }

    #[inline]
    pub fn get_mut(&mut self, bundle_id: BundleId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(bundle_id)
    }

    #[inline]
    pub fn get_2_mut(
        &mut self,
        i1: BundleId,
        i2: BundleId,
    ) -> Option<(&mut Archetype, &mut Archetype)> {
        self.archetypes.get_2_mut(i1, i2)
    }
}
