use std::{
    collections::HashMap,
    any::{ TypeId, type_name },
    mem::ManuallyDrop
};

use tar_ecs_macros::foreach_tuple;
use crate::{
    component::{ Component, Components, ComponentId },
    store::sparse::SparseSetIndex
};

/// Bundle is implemented for every Component, as well as for every tuple consisting of Components
/// 
/// SAFETY:
/// - Manual implementations are discouraged
pub unsafe trait Bundle: Send + Sync + 'static {
    fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId));
    fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8));
}

unsafe impl<T: Component> Bundle for T {
    fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId)) {
        func(components.init::<T>())
    }
    fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8)) {
        let mut temp = ManuallyDrop::new(self);
        func(*components.get_id_from::<T>().unwrap(), &mut temp as *mut ManuallyDrop<Self> as *mut u8)
    }
}

macro_rules! component_tuple_impl {
    ($($c:ident),*) => {
        unsafe impl<$($c: Bundle),*> Bundle for ($($c,)*) {
            #[allow(unused_variables)]
            fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId)) {
                $(<$c as Bundle>::component_ids(components, func);)*
            }
            #[allow(unused_variables, unused_mut)]
            fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8)) {
                #[allow(non_snake_case)]
                let ($(mut $c,)*) = self;
                $(
                    $c.get_components(components, &mut *func);
                )*
            }
        }
    };
}

foreach_tuple!(component_tuple_impl, 0, 15, B);


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BundleId(u32);

impl BundleId {
    #[inline]
    pub fn new(index: usize) -> Self {
        Self(index as u32)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl SparseSetIndex for BundleId {
    #[inline]
    fn from_usize(value: usize) -> Self {
        Self::new(value)
    }

    #[inline]
    fn as_usize(&self) -> usize {
        self.index()
    }
}


#[derive(Debug)]
pub struct BundleInfo {
    id: BundleId,
    components: Vec<ComponentId>
}

impl BundleInfo {
    #[inline]
    pub fn id(&self) -> BundleId {
        self.id
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ComponentId> {
        self.components.iter()
    }
}


#[derive(Debug)]
pub struct Bundles {
    bundles: Vec<BundleInfo>,
    indices: HashMap<TypeId, BundleId>
}

impl Bundles {
    #[inline]
    pub fn new() -> Self {
        Self {
            bundles: Vec::new(),
            indices: HashMap::new()
        }
    }

    #[inline]
    pub fn init<'a, T: Bundle>(&'a mut self, components: &mut Components) -> &'a BundleInfo {
        let id = self.indices.entry(TypeId::of::<T>()).or_insert_with(|| {
            let mut component_ids = Vec::new();
            T::component_ids(components, &mut |id| component_ids.push(id));
            let id = BundleId::new(self.bundles.len());
            let info = Self::_init(type_name::<T>(), component_ids, id);
            self.bundles.push(info);
            id
        });
        unsafe { self.bundles.get_unchecked(id.index()) }
    }

    #[inline]
    fn _init(name: &'static str, components: Vec<ComponentId>, id: BundleId) -> BundleInfo {
        let mut deduped = components.clone();
        deduped.sort();
        deduped.dedup();
        assert!(
            deduped.len() == components.len(),
            "Bundle {} has duplicate components",
            name
        );

        BundleInfo {
            id,
            components
        } 
    }

    #[inline]
    pub fn get_info(&self, id: BundleId) -> Option<&BundleInfo> {
        self.bundles.get(id.index())
    }

    #[inline]
    pub unsafe fn get_info_unchecked(&self, id: BundleId) -> &BundleInfo {
        self.bundles.get_unchecked(id.index())
    }

    #[inline]
    pub fn get_id(&self, id: TypeId) -> Option<BundleId> {
        self.indices.get(&id).cloned()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bundles.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &BundleInfo> {
        self.bundles.iter()
    }
}

