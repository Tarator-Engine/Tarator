use std::{
    sync::Arc,
    collections::HashMap
};
use parking_lot::{ Mutex, MutexGuard };

use crate::{
    component::{ ComponentId, Components },
    store::{ sparse::SparseSetIndex, table::Table },
    bundle::{ BundleInfo, BundleComponents }
};


/// Each [`Archetype`] gets its own unique [`ArchetypeId`]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    pub const EMPTY: Self = Self(0);

    /// Also marks [`EntityMeta`] as destroyed
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl SparseSetIndex for ArchetypeId {
    #[inline]
    fn as_usize(&self) -> usize {
        self.index()
    }

    #[inline]
    fn from_usize(value: usize) -> Self {
        Self::new(value)
    }
}


/// Stores an unique set of [`Component`](crate::component::Component)s ([`Bundle`]) and keeps all
/// stored [`Component`](crate::componen::Component) packed, with no empty indices between them.
///
/// Every function that returns an [`Entity`] relocated some [`Component`]s of another [`Entity`],
/// which means that its [`EntityMeta`](crate::entity::EntityMeta) must get updated!
#[derive(Debug)]
pub struct Archetype {
    id: ArchetypeId,
    parents: Vec<ArchetypeId>,
    table: Arc<Mutex<Table>>
}

impl Archetype {
    /// SAFETY:
    /// - Function should only be used for the empty ArchetypeId
    /// - Will mess with [`World`](crate::world::World) if above is not regarded
    #[inline]
    pub unsafe fn empty(id: ArchetypeId) -> Self {
        Self {
            id,
            parents: Vec::new(),
            table: Arc::new(Mutex::new(Table::empty()))
        }
    }

    /// SAFETY:
    /// - Should be called with `capacity` > 0, could else lead to possible problems
    #[inline]
    pub fn with_capacity<'a>(
        id: ArchetypeId,
        component_ids: impl Iterator<Item = &'a ComponentId>,
        components: &Components,
        capacity: usize
    ) -> Self {

        Self {
            id,
            parents: Vec::new(),
            table: Arc::new(Mutex::new(Table::with_capacity(component_ids, components, capacity)))
        }
    }

    #[inline]
    pub fn table(&self) -> Arc<Mutex<Table>> {
        self.table.clone()
    }

    #[inline]
    pub fn table_lock(&self) -> MutexGuard<Table> {
        self.table.lock()
    }

    #[inline]
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Checks if `archetype` is contains at least all [`Component`](crate::component::Component)s,
    /// and if so, add them to the list of parents.
    #[inline]
    pub fn init_parent(&mut self, parent: &Archetype) {

        // If `archetype` is already a set parent, ignore
        if self.parents.contains(&parent.id()) {
            return;
        }

        let self_table = self.table.lock();
        let parent_table = parent.table.lock();

        for component_id in self_table.component_ids() {
            // If the `archetype` does not contain every component_id of `self`, `archetype` is not
            // a parent. We only care about parents for now.
            if !parent_table.contains(component_id) {
                return;
            }
        }

        self.parents.push(parent.id());
    }

    #[inline]
    pub fn get_parent(&self, component_id: usize) -> Option<ArchetypeId> {
        self.parents.get(component_id).map(|v| *v)
    }

    #[inline]
    pub fn has_parents(&self) -> bool {
        self.parents.len() != 0
    }

    #[inline]
    pub fn parents(&self) -> impl Iterator<Item = &ArchetypeId> {
        self.parents.iter() 
    }
}


/// Manages all [`Archetype`]s of a [`World`](crate::world::World), as well as each one's parent
/// [`Archetype`]s.
#[derive(Debug)]
pub struct Archetypes {
    archetypes: Vec<Archetype>,
    archetype_ids: HashMap<BundleComponents, ArchetypeId>
}

impl Archetypes {
    #[inline]
    pub fn new() -> Self {
        // Entities with no components will be assigned this archetype
        let empty = unsafe { Archetype::empty(ArchetypeId::new(0)) };
        Self {
            archetypes: vec![empty],
            archetype_ids: HashMap::new()
        }
    }

    /// Automatically initializes all the parent [`ArchetypeId`]s in the new [`Archetype`], as well
    /// as in all the other [`Archetype`]s.
    pub fn create_with_capacity(
        &mut self,
        bundle_components: &BundleComponents,
        components: &Components,
        capacity: usize
    ) -> ArchetypeId {
        let id = ArchetypeId::new(self.len());
        let mut archetype = Archetype::with_capacity(id, bundle_components.iter(), components, capacity);

        // Check every current archetype and our newly created archetype if they are parents.
        for other_archetype in &mut self.archetypes {
            other_archetype.init_parent(&archetype);
            archetype.init_parent(&other_archetype);
        }

        self.archetypes.push(archetype);
        self.archetype_ids.insert(bundle_components.clone(), id);

        id
    }

    #[inline]
    pub fn get_id_from_components(&self, components: &BundleComponents) -> Option<ArchetypeId> {
        self.archetype_ids.get(components).map(|v| *v)
    }

    pub fn get_id_from_components_or_create_with_capacity(
        &mut self,
        components: &Components,
        bundle_components: &BundleComponents,
        capacity: usize
    ) -> ArchetypeId {
        self.get_id_from_components(bundle_components).unwrap_or_else(|| {
            self.create_with_capacity(bundle_components, components, capacity)
        })
    }

    #[inline]
    pub fn get_from_bundle(&self, info: &BundleInfo) -> Option<&Archetype> {
        let id = self.get_id_from_components(info.components())?;
        self.archetypes.get(id.index())
    }

    #[inline]
    pub fn get_from_bundle_mut(&mut self,info: &BundleInfo) -> Option<&mut Archetype> {
        let id = self.get_id_from_components(info.components())?;
        self.archetypes.get_mut(id.index())
    }

    #[inline]
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.index())
    }

    #[inline]
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.index())
    }

    #[inline]
    pub fn get_2_mut(&mut self, a: ArchetypeId, b: ArchetypeId) -> (&mut Archetype, &mut Archetype) {
        if a.index() > b.index() {
            let (b_slice, a_slice) = self.archetypes.split_at_mut(a.index());
            (&mut a_slice[0], &mut b_slice[b.index()])
        } else {
            let (a_slice, b_slice) = self.archetypes.split_at_mut(b.index());
            (&mut a_slice[a.index()], &mut b_slice[0])
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &Archetype {
        self.archetypes.get_unchecked(index)  
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Archetype {
        self.archetypes.get_unchecked_mut(index)  
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }
}

