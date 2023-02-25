use std::{
    alloc::Layout,
    any::{ Any, TypeId, type_name },
    sync::Arc,
    mem::needs_drop,
    collections::HashMap, 
    marker::PhantomData
};
use parking_lot::{Mutex, MutexGuard};

use crate::{
    store::{sparse::SparseSetIndex, table::Table},
    bundle::Bundle,
    archetype::{ Archetypes, ArchetypeId }
};

/// A [`Component`] is nothing more but data, which can be stored in a given
/// [`World`](crate::world::World) on an [`Entity`](crate::entity::Entity). [`Component`] can
/// manually be implemented on a type, or via `#[derive(Component)]`.
///
/// Read further: [`Bundle`]
pub trait Component: Send + Sync + 'static {}


/// Every [`Component`] gets its own [`ComponentId`] per [`World`](crate::world::World). This
/// [`ComponentId`] directly links to a [`ComponentDescription`], which contains some crutial
/// information about a [`Component`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ComponentId(u32);

impl ComponentId {
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl SparseSetIndex for ComponentId {
    #[inline]
    fn from_usize(value: usize) -> Self {
        Self::new(value)
    }

    #[inline]
    fn as_usize(&self) -> usize {
        self.index()
    }
}


/// Contains information about a [`Component`], most important its [`Layout`] and its [`Drop`]
/// function, if any, which is crutial for [`RawStore`](crate::store::raw_store::RawStore)to drop a
/// stored [`Component`] correctly, and not create any memory leaks because of some allocations
/// from a [`Component`].
pub struct ComponentDescription {
    name: &'static str,
    send_sync: bool,
    type_id: Option<TypeId>,
    layout: Layout,
    drop: Option<unsafe fn(*mut u8)>
}

impl std::fmt::Debug for ComponentDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentDescriptor")
            .field("name", &self.name)
            .field("send_sync", &self.send_sync)
            .field("type_id", &self.type_id)
            .field("layout", &self.layout)
            .field("drop", &match self.drop {
                Some(_) => "Some(_)",
                None => "None"
            })
            .finish()
    }
}

impl ComponentDescription {
    /// SAFETY:
    /// - `ptr` must be owned
    /// - `ptr` must point to valid data of type `T`
    #[inline]
    unsafe fn drop_ptr<T>(ptr: *mut u8) {
        ptr.cast::<T>().drop_in_place()
    }

    /// New [`ComponentDescription`] from given [`Component`]
    pub fn new<T: Component>() -> Self {
        Self {
            name: type_name::<T>(),
            send_sync: true,
            type_id: Some(TypeId::of::<T>()),
            layout: Layout::new::<T>(),
            drop: needs_drop::<T>().then_some(Self::drop_ptr::<T>)
        }
    }

    /// SAFETY:
    /// - `layout` and `drop` must correspond to the same type
    /// - type must be `Send + Sync`
    pub unsafe fn new_raw(
        name: impl Into<&'static str>,
        layout: Layout,
        drop: Option<unsafe fn(*mut u8)>
    ) -> Self {
        Self {
            name: name.into(),
            send_sync: true,
            type_id: None,
            layout,
            drop
        }
    }

    pub fn new_non_send_sync<T: Any>() -> Self {
        Self {
            name: type_name::<T>(),
            send_sync: false,
            type_id: Some(TypeId::of::<T>()),
            layout: Layout::new::<T>(),
            drop: needs_drop::<T>().then_some(Self::drop_ptr::<T>)
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

    #[inline]
    pub fn send_sync(&self) -> bool {
        self.send_sync
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        self.type_id
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns the drop function of this [`ComponentDescription`]'s [`Component`]
    #[inline]
    pub fn drop(&self) -> Option<unsafe fn(*mut u8)> {
        self.drop
    }
}


#[derive(Debug)]
pub struct Components {
    components: Vec<ComponentDescription>,
    indices: HashMap<TypeId, ComponentId>
}

impl Components {
    #[inline]
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            indices: HashMap::new()
        }
    }

    #[inline]
    pub fn init<T: Component>(&mut self) -> ComponentId {
        let Self { components, indices } = self;
        *indices.entry(TypeId::of::<T>()).or_insert_with(|| Self::_init(components, ComponentDescription::new::<T>()))
    }

    #[inline]
    pub fn init_from_description(&mut self, description: ComponentDescription) -> ComponentId {
        Self::_init(&mut self.components, description)
    }

    #[inline]
    fn _init(components: &mut Vec<ComponentDescription>, description: ComponentDescription) -> ComponentId {
        let id = ComponentId::new(components.len());
        components.push(description);
        id
    }

    #[inline]
    pub fn get_description(&self, id: ComponentId) -> Option<&ComponentDescription> {
        self.components.get(id.index())
    }

    #[inline]
    pub unsafe fn get_description_unchecked(&self, id: ComponentId) -> &ComponentDescription {
        self.components.get_unchecked(id.index())
    }

    #[inline]
    pub fn get_id(&self, id: TypeId) -> Option<&ComponentId> {
        self.indices.get(&id)
    }

    #[inline]
    pub fn get_id_from<T: Any>(&self) -> Option<&ComponentId> {
        self.get_id(TypeId::of::<T>())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ComponentDescription> {
        self.components.iter()
    }
}


/// An [`Iterator`] for a given [`Bundle`], which iterates over all
/// [`Archetype`](crate::archetype::Archetype)s of a [`World`](crate::world::World) who contain the
/// [`Bundle`].
#[derive(Debug)]
pub struct ComponentQuery<'a, T: Bundle> {
    ids: Vec<ComponentId>,
    tables: Vec<Arc<Mutex<Table>>>,
    index: usize,
    table: usize,
    marker: PhantomData<&'a T>
}

impl<'a, T: Bundle> ComponentQuery<'a, T> {
    pub fn new(
        archetype_ids: Vec<ArchetypeId>,
        archetypes: &'a mut Archetypes,
        components: &'a Components,
    ) -> Self {
        let mut tables = Vec::with_capacity(archetype_ids.len());

        for id in archetype_ids {
            if let Some(archetype) = archetypes.get(id) {
                tables.push(archetype.table())
            } else {
                debug_assert!(false, "Invalid Id was passed!");
            }
        }

        Self {
            ids: T::get_component_ids(components),
            tables,
            index: 0,
            table: 0,
            marker: PhantomData
        }
    }
}

impl<'a, T: Bundle> Iterator for ComponentQuery<'a, T> {
    type Item = T::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(table) = self.tables.get(self.table) {
            {
                let index = self.index;
                let table = table.lock();

                if table.len() > index {
                    self.index += 1;
                    return unsafe { Some(table.get_unchecked::<T>(&self.ids, index)) };
                }
            }

            self.table += 1;
            self.index = 0;
            return self.next();
        }
        
        None
    } 
}

/// An [`Iterator`] for a given [`Bundle`], which iterates mutably over all
/// [`Archetype`](crate::archetype::Archetype)s of a [`World`](crate::world::World) who contain the
/// [`Bundle`].
#[derive(Debug)]
pub struct ComponentQueryMut<'a, T: Bundle> {
    ids: Vec<ComponentId>,
    tables: Vec<Arc<Mutex<Table>>>,
    index: usize,
    table: usize,
    marker: PhantomData<&'a mut T>
}

impl<'a, T: Bundle> ComponentQueryMut<'a, T> {
    pub fn new(
        archetype_ids: Vec<ArchetypeId>,
        archetypes: &'a mut Archetypes,
        components: &'a Components,
    ) -> Self {
        let mut tables = Vec::with_capacity(archetype_ids.len());

        for id in archetype_ids {
            if let Some(archetype) = archetypes.get(id) {
                tables.push(archetype.table())
            } else {
                debug_assert!(false, "Invalid Id was passed!");
            }
        }

        Self {
            ids: T::get_component_ids(components),
            tables,
            index: 0,
            table: 0,
            marker: PhantomData
        }
    }
}

impl<'a, T: Bundle> Iterator for ComponentQueryMut<'a, T> {
    type Item = T::MutRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(table) = self.tables.get_mut(self.table) {
            {
                let index = self.index;
                let mut table = table.lock();

                if table.len() > index {
                    self.index += 1;
                    return unsafe { Some(table.get_unchecked_mut::<T>(&self.ids, index)) };
                }
            }

            self.table += 1;
            self.index = 0;
            self.next()
        } else {
            None
        }
    } 
}

