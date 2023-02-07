use std::collections::HashMap;

use crate::{
    component::{ ComponentId, Components },
    store::{
        sparse::{ SparseSetIndex, SparseSet, MutSparseSet },
        raw_store::RawStore
    },
    entity::Entity,
    bundle::{ BundleId, BundleInfo, Bundle }
};


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    pub const EMPTY: Self = Self(0);

    /// Also marks [`EntityMeta`] as destroyed
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub fn new(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub fn index(self) -> usize {
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


#[derive(Debug)]
pub struct Archetype {
    id: ArchetypeId,
    components: SparseSet<ComponentId, RawStore>,
    parents: Vec<ArchetypeId>,
    entities: Vec<Entity>,
}

impl Archetype {
    /// SAFETY:
    /// - Function should only be used for the empty ArchetypeId
    /// - Will mess with [`World`] if above is not regarded
    pub unsafe fn empty(id: ArchetypeId) -> Self {
        Self {
            id,
            components: SparseSet::new(),
            parents: Vec::new(),
            entities: Vec::new()
        }
    }

    pub fn with_capacity<'a>(
        id: ArchetypeId,
        component_ids: impl Iterator<Item = &'a ComponentId>,
        components: &Components,
        capacity: usize
    ) -> Self {
        let mut component_set = MutSparseSet::new();

        for component_id in component_ids {
            let info = components.get_info(*component_id).unwrap();
            let description = info.description();
            let store = unsafe { RawStore::with_capacity(description.layout(), description.drop(), capacity) };
            component_set.insert(info.id(), store);
        }

        Self {
            id,
            components: component_set.lock(),
            parents: Vec::new(),
            entities: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    pub fn set<'a, T: Bundle<'a>>(&mut self, components: &Components, entity: Entity, data: T) {
        self.entities.push(entity);
        data.get_components(components, &mut |id, component| {
            let store = self.components.get_mut(id).expect("Component is not part of this archetype!");
            unsafe { store.push(component); }
        });
    }

    /// SAFETY:
    /// - Function should only be used for the empty ArchetypeId
    /// - Will mess with [`World`] if above is not regarded
    #[inline]
    pub unsafe fn set_empty(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    #[inline]
    pub fn get<'a, T: Bundle<'a>>(&self, components: &Components, index: usize) -> T::Ref {
        if index >= self.len() {
            return T::EMPTY_REF;
        }

        // SAFETY:
        // Already bounds checked
        unsafe {
            T::from_components::<T>(components, &mut |id| {
                let raw_store = self.components.get(id)?;
                Some(raw_store.get_unchecked(index))
            })
        }
    }

    #[inline]
    pub fn get_mut<'a, T: Bundle<'a>>(&mut self, components: &Components, index: usize) -> T::MutRef {
        if index >= self.len() {
            return T::EMPTY_MUTREF;
        }

        // SAFETY:
        // Already bounds checked
        unsafe {
            T::from_components_mut::<T>(components, &mut |id| {
                let raw_store = self.components.get_mut(id)?;
                Some(raw_store.get_unchecked_mut(index))
            })
        }
    }

    #[inline]
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    #[inline]
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.components.contains(component_id)
    }

    #[inline]
    pub fn component_ids(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.components.indices()
    }

    #[inline]
    pub fn init_parent(&mut self, archetype: &Archetype) {

        // If `archetype` is already a set parent, ignore
        if self.parents.contains(&archetype.id()) {
            return;
        }

        for component_id in self.component_ids() {
            // If the `archetype` does not contain every component_id of `self`, `archetype` is not
            // a parent. We only care about parents for now.
            if !archetype.contains(component_id) {
                return;
            }
        }

        self.parents.push(archetype.id());
    }

    #[inline]
    pub fn is_empty_archetype(&self) -> bool {
        self.components.len() == 0
    }
}


#[derive(Debug)]
pub struct Archetypes {
    archetypes: Vec<Archetype>,
    archetype_ids: HashMap<BundleId, ArchetypeId>
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

    pub fn create_with_capacity(
        &mut self,
        bundle_info: &BundleInfo,
        components: &Components,
        capacity: usize
    ) -> ArchetypeId {
        let id = ArchetypeId::new(self.len());
        let mut archetype = Archetype::with_capacity(id, bundle_info.iter(), components, capacity);

        // Check every current archetype and our newly created archetype if they are parents.
        for other_archetype in &mut self.archetypes {
            other_archetype.init_parent(&archetype);
            archetype.init_parent(&other_archetype);
        }

        self.archetypes.push(archetype);
        self.archetype_ids.insert(bundle_info.id(), id);

        id
    }

    #[inline]
    pub fn get_from_bundle(&self, id: BundleId) -> Option<&Archetype> {
        let id = *self.archetype_ids.get(&id)?;
        self.archetypes.get(id.index())
    }

    #[inline]
    pub fn get_from_bundle_mut(&mut self, id: BundleId) -> Option<&mut Archetype> {
        let id = *self.archetype_ids.get(&id)?;
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

