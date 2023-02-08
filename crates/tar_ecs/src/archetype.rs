use std::collections::HashMap;

use crate::{
    component::{ ComponentId, Components },
    store::{
        sparse::{ SparseSetIndex, SparseSet, MutSparseSet },
        raw_store::RawStore
    },
    entity::Entity,
    bundle::{ BundleInfo, Bundle, BundleComponents }
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
    pub fn init<'a, T: Bundle<'a>>(&mut self, components: &Components, entity: Entity, data: T) {
        self.entities.push(entity);

        // SAFETY:
        // We initialize `component` in our store via [`RawStore::push`]
        unsafe {
            data.get_components(components, &mut |component_id, component| {
                let store = self.components.get_mut(component_id).expect("Component is not part of this archetype!");
                store.push(component);
            });
        }
    }

    #[inline]
    pub fn set<'a, T: Bundle<'a>>(&mut self, components: &Components, index: usize, data: T) {
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());
        // SAFETY:
        // We initialize `component` in our store via [`RawStore::push`]
        unsafe {
            data.get_components(components, &mut |component_id, component| {
                let store = self.components.get_mut(component_id).expect("Component is not part of this archetype!");
                store.replace_unchecked(index, component);
            });
        }
    }

    /// SAFETY:
    /// - Function should only be used for the empty ArchetypeId
    /// - Will mess with [`World`] if above is not regarded
    #[inline]
    pub unsafe fn set_empty(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    #[inline]
    pub fn get<'a, T: Bundle<'a>>(&self, components: &Components, index: usize) -> T::WrappedRef {
        if index >= self.len() {
            debug_assert!(false, "DEBUG: Index is out of bounds! ({}>={})", index, self.len());
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

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    /// - Returned references may be invalid
    #[inline]
    pub unsafe fn get_unchecked<'a, T: Bundle<'a>>(&self, components: &Components, index: usize) -> T::Ref {
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());

        unsafe {
            T::from_components_unchecked::<T>(components, &mut |id| {
                let raw_store = self.components.get(id).unwrap();
                raw_store.get_unchecked(index)
            })
        }
    }

    #[inline]
    pub fn get_mut<'a, T: Bundle<'a>>(&mut self, components: &Components, index: usize) -> T::WrappedMutRef {
        if index >= self.len() {
            debug_assert!(false, "DEBUG: Index is out of bounds! ({}>={})", index, self.len());
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

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    /// - Returned mutable references may be invalid
    #[inline]
    pub unsafe fn get_unchecked_mut<'a, T: Bundle<'a>>(&mut self, components: &Components, index: usize) -> T::MutRef {
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());

        unsafe {
            T::from_components_unchecked_mut::<T>(components, &mut |id| {
                let raw_store = self.components.get_mut(id).unwrap();
                raw_store.get_unchecked_mut(index)
            })
        }
    }

    #[inline]
    pub fn get_entity(&self, index: usize) -> Option<Entity> {
        self.entities.get(index).map(|entity| *entity)
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

    #[inline]
    pub fn is_empty_archetype(&self) -> bool {
        self.components.len() == 0
    }

    /// SAFETY:
    /// - Unset components in the parent have to be set immediately after this call
    /// - `index` has to be valid
    pub unsafe fn move_into_parent(&mut self, parent: &mut Archetype, index: usize) -> Option<Entity> {

        let is_last = index == self.len() - 1;

        let swapped_entity = if is_last {
            self.entities.pop();

            None
        } else {
            let entity = self.entities.swap_remove(index);

            Some(entity)
        };

        for (component_id, parent_raw_store) in parent.components.iter_mut() {
            if let Some(raw_store) = self.components.get_mut(*component_id) {
                    let ptr = raw_store.swap_remove_and_forget_unchecked(index);
                    parent_raw_store.push(ptr);
            }
        }

        swapped_entity
    }

    // SAFETY:
    // - Drops the components which are not in the child
    /// - `index` has to be valid
    pub unsafe fn move_into_child(&mut self, child: &mut Archetype, index: usize) -> Option<Entity> {
        let is_last = index == self.len() - 1;

        let swapped_entity = if is_last {
            self.entities.pop();

            None
        } else {
            let entity = self.entities.swap_remove(index);

            Some(entity)
        };
        
        for (component_id, raw_store) in self.components.iter_mut() {
            if let Some(child_raw_store) = child.components.get_mut(*component_id) {
                let ptr = raw_store.swap_remove_and_forget_unchecked(index);
                child_raw_store.push(ptr);
            } else {
                raw_store.swap_remove_and_drop_unchecked(index);
            }
        }

        swapped_entity
    }

    // SAFETY:
    // - Drops all components at `index`
    // - `index` has to be valid
    pub unsafe fn drop_entity(&mut self, index: usize) -> Option<Entity> {
        let is_last = index == self.len() - 1;

        let swapped_entity = if is_last {
            self.entities.pop();

            None
        } else {
            let entity = self.entities.swap_remove(index);

            Some(entity)
        };

        for (_, raw_store) in self.components.iter_mut() {
            raw_store.swap_remove_and_drop_unchecked(index);
        }

        swapped_entity
    }
}


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

