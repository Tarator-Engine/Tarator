use std::{collections::HashSet, any, mem, ops::{Add, Sub}};

use crate::{
    component::{ Component, ComponentId },
    type_info::TypeInfo,
};
use fxhash::FxBuildHasher;
use tar_ecs_macros::{ foreach_tuple, identifier };


/// Bundle is implemented for every type implementing[`Component`], as well as for every tuple
/// consisting [`Bundle`]s. This means that a tuple of multiple [`Component`]s is also a
/// [`Bundle`].
///
/// This is isefull to set and get [`Component`]s on a [`World`](crate::world::World), as setting
/// or deleting every [`Component`] one by one can get a bit slow.
///
/// SAFETY:
/// - Manual implementations are discouraged
pub unsafe trait Bundle: Sized + Send + Sync + 'static {
    /// Implemented as a tuple of [`Component`] refs/ptr
    type Ptr: Copy;
    type Ref<'a>: 'a + Copy;
    type Mut<'a>: 'a;

    #[inline]
    fn uid() -> UBundleId {
        unsafe { mem::transmute(any::TypeId::of::<Self>()) }
    }

    /// Initializes and gets the [`ComponentId`]s via `func`
    fn init_component_ids(type_info: &mut impl TypeInfo, func: &mut impl FnMut(ComponentId));

    unsafe fn from_components(
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId) -> *mut u8,
    ) -> Self::Ptr;

    /// Returns a tuple of references to the components in the order of `Self`. The references are
    /// set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components_as_ref<'a>(
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId) -> Option<*const u8>,
    ) -> Option<Self::Ref<'a>>;

    unsafe fn from_components_as_mut<'a>(
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId) -> Option<*mut u8>,
    ) -> Option<Self::Mut<'a>>;

    /// Get the components of this [`Bundle`] with a corresponding [`ComponentId`]. This passes
    /// ownership to `func`.
    ///
    /// SAFETY:
    /// - pointer in `func` must be used, else will create memory leak if data has to be dropped
    /// - data in `func` must be manually dropped
    unsafe fn get_components(
        self,
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId, *mut u8),
    );
}

/// Helper-trait to clone [`Bundle`]s with ease, where every [`Component`] has [`Clone`]
/// implemented
pub unsafe trait CloneBundle: Bundle + Clone {
    fn clone_bundles<'a>(bundles: Self::Ref<'a>) -> Self;
}


unsafe impl<T: Component> Bundle for T {
    type Ptr = *mut Self;
    type Ref<'a> = &'a Self;
    type Mut<'a> = &'a mut Self;

    #[inline]
    fn init_component_ids(type_info: &mut impl TypeInfo, func: &mut impl FnMut(ComponentId)) {
        func(type_info.init_component_from::<T>())
    }

    #[inline]
    unsafe fn from_components(
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId) -> *mut u8,
    ) -> Self::Ptr {
        func(
            type_info
                .get_component_id_from::<T>()
                .expect("Component not initialized!"),
        )
        .cast::<Self>()
    }

    unsafe fn from_components_as_ref<'a>(
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId) -> Option<*const u8>,
    ) -> Option<Self::Ref<'a>> {
        func(type_info.get_component_id_from::<T>()?).map(|data| &*data.cast::<T>())
    }

    unsafe fn from_components_as_mut<'a>(
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId) -> Option<*mut u8>,
    ) -> Option<Self::Mut<'a>> {
        func(type_info.get_component_id_from::<T>()?).map(|data| &mut *data.cast::<T>())
    }

    #[inline]
    unsafe fn get_components(
        self,
        type_info: &impl TypeInfo,
        func: &mut impl FnMut(ComponentId, *mut u8),
    ) {
        func(
            type_info
                .get_component_id_from::<T>()
                .expect("Component not initialized!"),
            (&mut mem::ManuallyDrop::new(self)) as *mut _ as *mut u8,
        )
    }
}

unsafe impl<T: Component + Clone> CloneBundle for T {
    fn clone_bundles<'a>(bundles: Self::Ref<'a>) -> Self {
        (*bundles).clone()
    }
}

macro_rules! component_tuple_impl {
    ($($c:ident),*) => {
        unsafe impl<$($c: Component + Bundle),*> Bundle for ($($c,)*) {
            type Ptr = ($($c::Ptr,)*);
            type Ref<'a> = ($($c::Ref<'a>,)*);
            type Mut<'a> = ($($c::Mut<'a>,)*);

            #[inline]
            #[allow(unused_variables)]
            fn init_component_ids(type_info: &mut impl TypeInfo, func: &mut impl FnMut(ComponentId)) {
                $(<$c as Bundle>::init_component_ids(type_info, func);)*
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components(type_info: &impl TypeInfo, func: &mut impl FnMut(ComponentId) -> *mut u8) -> Self::Ptr {
                ($($c::from_components(type_info, func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_as_ref<'a>(type_info: &impl TypeInfo, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Option<Self::Ref<'a>> {
                Some(($($c::from_components_as_ref(type_info, func)?,)*))
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_as_mut<'a>(type_info: &impl TypeInfo, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Option<Self::Mut<'a>> {
                Some(($($c::from_components_as_mut(type_info, func)?,)*))
            }

            #[inline]
            #[allow(unused_variables, unused_mut)]
            unsafe fn get_components(self, type_info: &impl TypeInfo, func: &mut impl FnMut(ComponentId, *mut u8)) {
                #[allow(non_snake_case)]
                let ($(mut $c,)*) = self;
                $($c.get_components(type_info, func);)*
            }
        }

        unsafe impl<$($c: Component + Clone),*> CloneBundle for ($($c,)*) {
            fn clone_bundles<'a>(bundles: Self::Ref<'a>) -> Self {
                #[allow(non_snake_case)]
                let ($($c,)*) = bundles;
                ($((*$c).clone(),)*)
            }
        }
    };
}

foreach_tuple!(component_tuple_impl, 1, 15, T);

identifier!(BundleId, u32);
identifier!(UBundleId, u64);


#[derive(Clone, Debug, PartialEq)]
pub struct BundleInfo {
    component_ids: HashSet<ComponentId, FxBuildHasher>,
}

impl BundleInfo {
    #[inline]
    pub const fn new(component_ids: HashSet<ComponentId, FxBuildHasher>) -> Self {
        Self { component_ids }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.component_ids.len()
    }

    #[inline]
    pub const fn component_ids(&self) -> &HashSet<ComponentId, FxBuildHasher> {
        &self.component_ids
    }

    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.component_ids.is_subset(&other.component_ids)
    }

    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        self.component_ids.is_superset(&other.component_ids)
    }
}

impl Add for &BundleInfo {
    type Output = BundleInfo;

    fn add(self, rhs: Self) -> Self::Output {
        let mut set = self.component_ids.clone();
        set.extend(rhs.component_ids.clone());

        BundleInfo::new(set)
    }
}

impl Sub for &BundleInfo {
    type Output = BundleInfo;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = HashSet::default();

        for x in self.component_ids().difference(rhs.component_ids()) {
            ret.insert(*x);
        }

        BundleInfo::new(ret)
    }
}

