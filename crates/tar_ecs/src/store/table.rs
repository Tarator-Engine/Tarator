use std::ops::{Deref, DerefMut};

use crate::{
    bundle::Bundle,
    component::{ComponentHashId, ComponentId, Components},
    entity::Entity,
    store::{
        raw_store::RawStore,
        sparse::{MutSparseSet, SparseSet},
    },
};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct Table {
    components: SparseSet<ComponentId, RawStore>,
    entities: Vec<Entity>,
}

impl Table {
    #[inline]
    pub unsafe fn empty() -> Self {
        Self {
            components: SparseSet::new(),
            entities: Vec::new(),
        }
    }

    #[inline]
    pub unsafe fn set_empty(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}

impl Table {
    pub fn with_capacity<'a>(
        component_ids: impl Iterator<Item = &'a ComponentId>,
        capacity: usize,
    ) -> Self {
        let mut component_set = MutSparseSet::new();

        for component_id in component_ids {
            Components::get_info(*component_id, |info| {
                let store =
                    unsafe { RawStore::with_capacity(info.layout(), info.drop(), capacity) };
                component_set.insert(*component_id, store);
            });
        }

        Self {
            components: component_set.lock(),
            entities: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn init<T: Bundle>(&mut self, entity: Entity, data: T) {
        self.entities.push(entity);

        let len = self.len();

        // SAFETY:
        // We initialize `component` in our store via [`RawStore::push()`]
        unsafe {
            data.get_components(&mut |component_id, component| {
                let store = self
                    .components
                    .get_mut(component_id)
                    .expect("Component is not part of this archetype!");

                // If the [`Component`] already has been initialized, drop/replace the last index
                if store.len() == len {
                    store.replace_unchecked(len - 1, component);
                } else {
                    store.push(component);
                }
            });
        }
    }

    #[inline]
    pub fn set<T: Bundle>(&mut self, index: usize, data: T) {
        debug_assert!(
            index < self.len(),
            "Index is out of bounds! ({}>={})",
            index,
            self.len()
        );

        // SAFETY:
        // We initialize `component` in our store via [`RawStore::push`]
        unsafe {
            data.get_components(&mut |component_id, component| {
                let store = self
                    .components
                    .get_mut(component_id)
                    .expect("Component is not part of this archetype!");
                store.replace_unchecked(index, component);
            });
        }
    }

    pub unsafe fn foreach_at(&mut self, index: usize, func: impl Fn(ComponentId, *mut u8)) {
        debug_assert!(
            index < self.len(),
            "Index is out of bounds! ({}>={})",
            index,
            self.len()
        );

        for (id, store) in self.components.iter_mut() {
            let data = store.get_unchecked_mut(index);
            func(*id, data);
        }
    }

    /// Performs a swap_remove and moves the removed into the parent. The returned [`Entity`] is
    /// the [`Entity`] that may have been relocated by the process.
    ///
    /// SAFETY:
    /// - Unset components in the parent have to be set immediately after this call
    /// - `index` has to be valid
    #[must_use = "The returned variant may contain a relocated Entity!"]
    pub unsafe fn move_into_parent(
        &mut self,
        parent: &mut RwLockWriteGuard<Self>,
        index: usize,
    ) -> Option<Entity> {
        debug_assert!(
            index < self.len(),
            "Index is out of bounds! ({}>={})",
            index,
            self.len()
        );

        let is_last = index == self.len() - 1;

        let swapped_entity = if is_last {
            self.entities.pop();

            None
        } else {
            self.entities.swap_remove(index);

            // SAFETY:
            // We just moved in a new entity
            Some(unsafe { *self.entities.get_unchecked(index) })
        };

        for (component_id, parent_raw_store) in parent.components.iter_mut() {
            if let Some(raw_store) = self.components.get_mut(*component_id) {
                let ptr = raw_store.swap_remove_and_forget_unchecked(index);
                parent_raw_store.push(ptr);
            }
        }

        swapped_entity
    }

    /// Performs a swap_remove and moves the removed into the child. The returned [`Entity`] is
    /// the [`Entity`] that may have been relocated by the process.
    ///
    // SAFETY:
    // - Drops the components which are not in the child
    /// - `index` has to be valid
    pub unsafe fn move_into_child(
        &mut self,
        child: &mut RwLockWriteGuard<Self>,
        index: usize,
    ) -> Option<Entity> {
        debug_assert!(
            index < self.len(),
            "Index is out of bounds! ({}>={})",
            index,
            self.len()
        );

        let is_last = index == self.len() - 1;

        let swapped_entity = if is_last {
            self.entities.pop();

            None
        } else {
            self.entities.swap_remove(index);

            // SAFETY:
            // We just moved in a new entity
            Some(unsafe { *self.entities.get_unchecked(index) })
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

    /// Performs a swap_remove. The returned [`Entity`] is the [`Entity`] that may have been
    /// relocated by the process.
    ///
    // SAFETY:
    // - Drops all components at `index`
    // - `index` has to be valid
    pub unsafe fn drop_entity(&mut self, index: usize) -> Option<Entity> {
        debug_assert!(
            index < self.len(),
            "Index is out of bounds! ({}>={})",
            index,
            self.len()
        );

        let is_last = index == self.len() - 1;

        let swapped_entity = if is_last {
            self.entities.pop();

            None
        } else {
            self.entities.swap_remove(index);

            // SAFETY:
            // We just moved in a new entity
            Some(unsafe { *self.entities.get_unchecked(index) })
        };

        for (_, raw_store) in self.components.iter_mut() {
            raw_store.swap_remove_and_drop_unchecked(index);
        }

        swapped_entity
    }
}

impl Table {
    /// SAFETY:
    /// - `index` has to be valid and in bounds
    #[inline]
    pub unsafe fn get<'a, T: Bundle>(&self, index: usize) -> Option<T::Ref<'a>> {
        T::some_ref_or_none(T::from_components(&mut |id| {
            let raw_store = self.components.get(id)?;
            Some(raw_store.get_unchecked(index))
        }))
    }

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    /// - Returned references may be invalid
    #[inline]
    pub unsafe fn get_unchecked<'a, T: Bundle>(&self, index: usize) -> T::Ref<'a> {
        T::from_components_unchecked(&mut |id| {
            let raw_store = self.components.get(id).unwrap();
            raw_store.get_unchecked(index)
        })
    }

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    #[inline]
    pub unsafe fn get_mut<'a, T: Bundle>(&mut self, index: usize) -> Option<T::Mut<'a>> {
        T::some_mut_or_none(T::from_components_mut(&mut |id| {
            let raw_store = self.components.get_mut(id)?;
            Some(raw_store.get_unchecked_mut(index))
        }))
    }

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    /// - Returned mutable references may be invalid
    #[inline]
    pub unsafe fn get_unchecked_mut<'a, T: Bundle>(&mut self, index: usize) -> T::Mut<'a> {
        T::from_components_unchecked_mut(&mut |id| {
            let raw_store = self.components.get_mut(id).unwrap();
            raw_store.get_unchecked_mut(index)
        })
    }

    pub unsafe fn get_unchecked_raw(
        &self,
        component: ComponentHashId,
        index: usize,
    ) -> Option<*const u8> {
        let id = Components::get_id(component)?;
        let raw_store = self.components.get(id)?;

        Some(raw_store.get_unchecked(index))
    }

    pub unsafe fn get_unchecked_mut_raw(
        &mut self,
        component: ComponentHashId,
        index: usize,
    ) -> Option<*mut u8> {
        let id = Components::get_id(component)?;
        let raw_store = self.components.get_mut(id)?;

        Some(raw_store.get_unchecked_mut(index))
    }
}

impl Table {
    #[inline]
    pub fn get_entity(&self, index: usize) -> Option<Entity> {
        self.entities.get(index).map(|entity| *entity)
    }

    #[inline]
    pub fn entities(&self) -> Vec<Entity> {
        self.entities.clone()
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
    pub fn is_empty_table(&self) -> bool {
        self.components.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }
}

pub struct TRef<'a, T: Bundle> {
    data: T::Ref<'a>,
    #[allow(unused)]
    table: RwLockReadGuard<'a, Table>,
}

impl<'a, T: Bundle> TRef<'a, T> {
    pub fn new(data: T::Ref<'a>, table: RwLockReadGuard<'a, Table>) -> Self {
        Self { data, table }
    }
}

impl<'a, T: Bundle> Deref for TRef<'a, T> {
    type Target = T::Ref<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct TMut<'a, T: Bundle> {
    data: T::Mut<'a>,
    #[allow(unused)]
    table: RwLockWriteGuard<'a, Table>,
}

impl<'a, T: Bundle> TMut<'a, T> {
    pub fn new(data: T::Mut<'a>, table: RwLockWriteGuard<'a, Table>) -> Self {
        Self { data, table }
    }
}

impl<'a, T: Bundle> Deref for TMut<'a, T> {
    type Target = T::Mut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: Bundle> DerefMut for TMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
