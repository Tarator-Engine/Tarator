use std::{
    alloc::Layout,
    any::{ Any, TypeId, type_name },
    mem::needs_drop, collections::HashMap
};

use crate::store::sparse::SparseSetIndex;

/// A `Component` is just data. Additionally, an [`Entity`] is just a redirection to a set of
/// multiple `Component`s.
pub trait Component: Send + Sync + 'static {}


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ComponentId(u32);

impl ComponentId {
    #[inline]
    pub fn new(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub fn index(self) -> usize {
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
    /// - `layout` and `drop` correspond to the same type
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

    #[inline]
    pub fn drop(&self) -> Option<unsafe fn(*mut u8)> {
        self.drop
    }
}


#[derive(Debug)]
pub struct ComponentInfo {
    id: ComponentId,
    description: ComponentDescription
}

impl ComponentInfo {
    #[inline]
    pub fn new(id: ComponentId, description: ComponentDescription) -> Self {
        Self { id, description }
    }
    
    #[inline]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    #[inline]
    pub fn description(&self) -> &ComponentDescription {
        &self.description
    }
}


#[derive(Debug)]
pub struct Components {
    components: Vec<ComponentInfo>,
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
    fn _init(components: &mut Vec<ComponentInfo>, description: ComponentDescription) -> ComponentId {
        let id = ComponentId::new(components.len());
        components.push(ComponentInfo::new(id, description));
        id
    }

    #[inline]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.components.get(id.index())
    }

    #[inline]
    pub unsafe fn get_info_unchecked(&self, id: ComponentId) -> &ComponentInfo {
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
    pub fn iter(&self) -> impl Iterator<Item = &ComponentInfo> {
        self.components.iter()
    }
}

