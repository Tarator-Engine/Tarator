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

/// Bundle is implemented for every type implementing[`Component`], as well as for every tuple
/// consisting [`Bundle`]s.
/// 
/// SAFETY:
/// - Manual implementations are discouraged
pub unsafe trait Bundle<'a>: Send + Sync + 'static {
    /// Implemented as a tuple of [`Component`] references wrapped in [`Option`]
    type Ref;

    /// Implemented as a tuple of mutable [`Component`] references wrapped in [`Option`]
    type MutRef;

    /// Implemented as a tuple of [`None`] values, used to return from a function, if for example an
    /// [`Entity`] doesn't exist.
    const EMPTY_REF: Self::Ref;

    /// Implemented as a tuple of [`None`] values, used to return from a function, if for example an
    /// [`Entity`] doesn't exist.
    const EMPTY_MUTREF: Self::MutRef;

    /// Initializes and gets the [`ComponentId`]s via `func`.
    fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId));

    /// Returns a tuple of references to the components in the order of `Self`. The references are
    /// set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components<T>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::Ref;

    /// Returns a tuple of mutable references to the components in the order of `Self`. The mutable
    /// references are set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components_mut<T>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::MutRef;

    /// Get the components of this [`Bundle`] with a corresponding [`ComponentId`]. This passes
    /// ownership to `func`.
    ///
    /// SAFETY:
    /// - pointer in `func` must be used, else will create memory leak if data has to be dropped
    /// - data in `func` must be manually dropped
    unsafe fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8));
}

unsafe impl<'a, C: Component> Bundle<'a> for C {
    type Ref = Option<&'a Self>;
    type MutRef = Option<&'a mut Self>;

    const EMPTY_REF: Self::Ref = None;
    const EMPTY_MUTREF: Self::MutRef = None;

    fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId)) {
        func(components.init::<C>())
    }

    unsafe fn from_components<T>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::Ref {
        Some(&*func(*components.get_id_from::<C>()?)?.cast::<Self>())
    }

    unsafe fn from_components_mut<T>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::MutRef {
        Some(&mut *func(*components.get_id_from::<C>()?)?.cast::<Self>())
    }

    unsafe fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8)) {
        func(*components.get_id_from::<C>().unwrap(), &mut ManuallyDrop::new(self) as *mut ManuallyDrop<Self> as *mut u8)
    }
}


macro_rules! component_tuple_impl {
    ($($c:ident),*) => {
        unsafe impl<'a, $($c: Bundle<'a>),*> Bundle<'a> for ($($c,)*) {
            type Ref = ($($c::Ref,)*);
            type MutRef = ($($c::MutRef,)*);

            const EMPTY_REF: Self::Ref = ($($c::EMPTY_REF,)*);
            const EMPTY_MUTREF: Self::MutRef = ($($c::EMPTY_MUTREF,)*);

            #[allow(unused_variables)]
            fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId)) {
                $(<$c as Bundle>::component_ids(components, func);)*
            }

            #[allow(unused_variables)]
            unsafe fn from_components<T>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::Ref {
                ($($c::from_components::<$c>(components, func),)*)
            }


            #[allow(unused_variables)]
            unsafe fn from_components_mut<T>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::MutRef {
                ($($c::from_components_mut::<$c>(components, func),)*)
            }


            #[allow(unused_variables, unused_mut)]
            unsafe fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8)) {
                #[allow(non_snake_case)]
                let ($(mut $c,)*) = self;
                $(
                    $c.get_components(components, func);
                )*
            }
        }
    };
}

foreach_tuple!(component_tuple_impl, 0, 15, B);


/// Every [`Bundle`] variation gets its own [`BundleId`], which gets managed by [`Bundles`]. It is
/// also used the map to an [`ArchetypeId`], which points to an [`Archetype`] with an _exact_ same
/// set of [`Component`]s.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BundleId(usize);

impl BundleId {
    #[inline]
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0
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


#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BundleComponents {
    components: Vec<ComponentId>
}

impl BundleComponents {
    #[inline]
    pub fn new(components: Vec<ComponentId>) -> Self {
        Self { components }
    }

    #[inline]
    pub fn insert(&mut self, mut components: Vec<ComponentId>) {
        self.components.append(&mut components);
        // let old_len = self.components.len();
        self.components.sort();
        self.components.dedup();
        /*assert!(
            self.components.len() == old_len,
            "Bundle {:#?} has duplicate components",
            self.components
        );*/ // May not need to check for duplicates in this case
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ComponentId> {
        self.components.iter()
    }
}

impl From<Vec<ComponentId>> for BundleComponents {
    fn from(value: Vec<ComponentId>) -> Self {
        Self::new(value)
    } 
}


/// Contains both the [`BundleId`] and [`ComponentId`]s of a [`Bundle`]. [`Bundle`]s with same sets
/// of [`Component`]s will still have the same [`BundleInfo`] as well as [`BundleId`].
#[derive(Debug)]
pub struct BundleInfo {
    id: BundleId,
    components: BundleComponents
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

    #[inline]
    pub fn components(&self) -> &BundleComponents {
        &self.components
    }
}


/// Manages every [`Bundle`] that gets used on a [`World`].
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

    /// Initializes given [`Bundle`], as well as all of its [`Component`]s.
    #[inline]
    pub fn init<'a, 'b, T: Bundle<'b>>(&'a mut self, components: &mut Components) -> &'a BundleInfo {
        let id = self.indices.entry(TypeId::of::<T>()).or_insert_with(|| {
            let mut component_ids = Vec::new();
            T::component_ids(components, &mut |id| component_ids.push(id));
            let id = BundleId::new(self.bundles.len());
            let info = Self::_init(type_name::<T>(), component_ids, id);
            self.bundles.push(info);
            id
        });
        // SAFETY:
        // Already initialized or inserted
        unsafe { self.bundles.get_unchecked(id.index()) }
    }
    
    #[inline]
    fn _init(name: &'static str, mut components: Vec<ComponentId>, id: BundleId) -> BundleInfo {
        let old_len = components.len();
        components.sort();
        components.dedup();
        assert!(
            components.len() == old_len,
            "Bundle {} has duplicate components",
            name
        );

        BundleInfo {
            id,
            components: BundleComponents::new(components)
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

