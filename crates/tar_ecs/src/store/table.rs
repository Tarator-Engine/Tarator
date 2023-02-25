use parking_lot::MutexGuard;
use crate::{
    bundle::Bundle,
    component::{ ComponentId, Components },
    store::{
        sparse::{ SparseSet, MutSparseSet },
        raw_store::RawStore
    },
    entity::Entity
};

#[derive(Debug)]
pub struct Table {
    components: SparseSet<ComponentId, RawStore>,
    entities: Vec<Entity>
}

impl Table {
    #[inline]
    pub unsafe fn empty() -> Self {
        Self {
            components: SparseSet::new(),
            entities: Vec::new()
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
        components: &Components,
        capacity: usize
    ) -> Self {
        let mut component_set = MutSparseSet::new();

        for component_id in component_ids {
            let description = components.get_description(*component_id).unwrap();
            let store = unsafe { RawStore::with_capacity(description.layout(), description.drop(), capacity) };
            component_set.insert(*component_id, store);
        }

        Self {
            components: component_set.lock(),
            entities: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    pub fn init<T: Bundle>(
        &mut self,
        components: &Vec<ComponentId>, 
        entity: Entity,
        data: T
    ) {
        self.entities.push(entity);

        let len = self.len();

        // SAFETY:
        // We initialize `component` in our store via [`RawStore::push()`]
        unsafe {
            data.get_components::<0>(components, &mut |component_id, component| {
                let store = self.components.get_mut(component_id).expect("Component is not part of this archetype!");

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
    pub fn set<T: Bundle>(
        &mut self,
        components: &Vec<ComponentId>,
        index: usize, 
        data: T
    ) {
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());

        // SAFETY:
        // We initialize `component` in our store via [`RawStore::push`]
        unsafe {
            data.get_components::<0>(components, &mut |component_id, component| {
                let store = self.components.get_mut(component_id).expect("Component is not part of this archetype!");
                store.replace_unchecked(index, component);
            });
        }
    }

    /// Performs a swap_remove and moves the removed into the parent. The returned [`Entity`] is
    /// the [`Entity`] that may have been relocated by the process.
    ///
    /// SAFETY:
    /// - Unset components in the parent have to be set immediately after this call
    /// - `index` has to be valid
    #[must_use = "The returned variant may contain a relocated Entity!"]
    pub unsafe fn move_into_parent(&mut self, parent: &mut MutexGuard<Self>, index: usize) -> Option<Entity> {
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());

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
    pub unsafe fn move_into_child(&mut self, child: &mut MutexGuard<Self>, index: usize) -> Option<Entity> {
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());

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
        debug_assert!(index < self.len(), "Index is out of bounds! ({}>={})", index, self.len());

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
    pub unsafe fn get<'a, T: Bundle>(
        &self,
        components: &Vec<ComponentId>,
        index: usize
    ) -> T::WrappedRef<'a> {
        T::from_components::<0>(components, &mut |id| {
            let raw_store = self.components.get(id)?;
            Some(raw_store.get_unchecked(index))
        })
    }

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    /// - Returned references may be invalid
    #[inline]
    pub unsafe fn get_unchecked<'a, T: Bundle>(
        &self,
        components: &Vec<ComponentId>,
        index: usize
    ) -> T::Ref<'a> {
        T::from_components_unchecked::<0>(components, &mut |id| {
            let raw_store = self.components.get(id).unwrap();
            raw_store.get_unchecked(index)
        })
    }

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    #[inline]
    pub unsafe fn get_mut<'a, T: Bundle>(
        &mut self,
        components: &Vec<ComponentId>,
        index: usize
    ) -> T::WrappedMutRef<'a> {
        T::from_components_mut::<0>(components, &mut |id| {
            let raw_store = self.components.get_mut(id)?;
            Some(raw_store.get_unchecked_mut(index))
        })
    }

    /// SAFETY:
    /// - `index` has to be valid and in bounds
    /// - Returned mutable references may be invalid
    #[inline]
    pub unsafe fn get_unchecked_mut<'a, T: Bundle>(
        &mut self,
        components: &Vec<ComponentId>,
        index: usize
    ) -> T::MutRef<'a> {
        T::from_components_unchecked_mut::<0>(components, &mut |id| {
            let raw_store = self.components.get_mut(id).unwrap();
            raw_store.get_unchecked_mut(index)
        })
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

