use std::collections::HashMap;

use crate::{
    component::{ ComponentId, Components, Component },
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


/// Defines a relationship between [`Archetype`]s. See [`Archetype.edges`] to get a better glance
/// at how it works with components
#[derive(Debug)]
pub struct Edge(ArchetypeId, ArchetypeId);

impl Edge {
    #[inline]
    pub fn new(add: ArchetypeId, remove: ArchetypeId) -> Self {
        Self(add, remove)
    }

    #[inline]
    pub fn add(&self) -> ArchetypeId {
        self.0
    }

    #[inline]
    pub fn remove(&self) -> ArchetypeId {
        self.0
    }
}


#[derive(Debug)]
pub struct Archetype {
    id: ArchetypeId,
    components: SparseSet<ComponentId, RawStore>,
    edges: HashMap<ComponentId, Edge>,
    entities: Vec<Entity>,
}

impl Archetype {
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
            edges: HashMap::new(),
            entities: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    pub fn set<T: Bundle>(&mut self, components: &Components, entity: Entity, data: T) {
        self.entities.push(entity);
        data.get_components(components, &mut |id, component| {
            let store = self.components.get_mut(id).expect("Component is not part of this archetype!");
            unsafe { store.push(component); }
        });
    }

    #[inline]
    pub fn get<T: Component>(&self, components: &Components, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        let component_id = *components.get_id_from::<T>()?;
        let raw_store = self.components.get(component_id)?;
        unsafe { Some(&*raw_store.get_unchecked(index).cast::<T>()) }
    }

    #[inline]
    pub fn get_mut<T: Component>(&mut self, components: &Components, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

        let component_id = *components.get_id_from::<T>()?;
        let raw_store = self.components.get_mut(component_id)?;
        unsafe { Some(&mut *raw_store.get_unchecked_mut(index).cast::<T>()) }
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
    pub fn insert_edge(&mut self, component_id: ComponentId, edge: Edge) {
        self.edges.insert(component_id, edge);
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
        Self {
            archetypes: Vec::new(),
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
        let archetype = Archetype::with_capacity(id, bundle_info.iter(), components, capacity);

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

