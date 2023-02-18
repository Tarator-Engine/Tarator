//! Some portions of this code where looked up from
//! <https://docs.rs/bevy_ecs/latest/src/bevy_ecs/bundle.rs.html>


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
/// consisting [`Bundle`]s. This means that a tuple of multiple [`Component`]s is also a
/// [`Bundle`].
/// 
/// # Examples
///
/// ```ignore
/// #[derive(Component)]
/// struct MyComponent(u64);
///
/// #[derive(Component)]
/// struct YourComponent(u32);
/// ```
///
/// Both structs are [`Component`]s now, which results in them also being [`Bundle`]s. But there's
/// more:
///
/// ```ignore
/// (MyComponent, YourComponent)
/// (YourComponent, MyComponent)
/// ```
///
/// These tuples are also [`Bundle`]s.
///
/// This is isefull to set and get [`Component`]s on a [`World`](crate::world::World), as setting
/// or deleting every [`Component`] one by one can get a bit slow. It also following generics work:
///
/// ```ignore
/// //                       \               HERE                 /
/// world.entity_set(entity, (MyComponent(42), YourComponent(16)) );
/// ```
///
/// ```ignore
/// //                                         \            HERE             /
/// let (mine, yours) = world.entity_get_mut::<(MyComponent, YourComponent)>(entity);
/// ```
/// 
/// ```ignore
/// //                                 \           HERE             /
/// for entity in world.entity_query::<(MyComponent, YourComponent)>() {
/// ```
///
/// ```ignore
/// //                                          \             HERE           /
/// for (yours, mine) in world.component_query::<(YourComponent, MyComponent)>(entity) {
/// ```
///
/// SAFETY:
/// - Manual implementations are discouraged
/// - [`Bundle::WrappedRef`] and [`Bundle::WrappedMutRef`] are supposed to be wrapped in[`Option`]
pub unsafe trait Bundle: Send + Sync + 'static {
    /// Implemented as a tuple of [`Component`] references
    type Ref<'a>;

    /// Implemented as a tuple of mutable [`Component`] references
    type MutRef<'a>;

    /// Implemented as a tuple of [`Component`] references wrapped in [`Option`]
    type WrappedRef<'a>;

    /// Implemented as a tuple of mutable [`Component`] references wrapped in [`Option`]
    type WrappedMutRef<'a>;

    /// Returns a tuple of [`None`] values, used to return from a function, if for example an
    /// [`Entity`] doesn't exist.
    fn empty_ref<'a>() -> Self::WrappedRef<'a>;

    /// Returns a tuple of [`None`] values, used to return from a function, if for example an
    /// [`Entity`] doesn't exist.
    fn empty_mut_ref<'a>() -> Self::WrappedMutRef<'a>;

    /// Initializes and gets the [`ComponentId`]s via `func`.
    fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId));

    /// Returns a tuple of references to the components in the order of `Self`. The references are
    /// set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::WrappedRef<'a>;

    /// Returns a tuple of references to the components in the order of `Self`. The references are
    /// set using the return value of `func`.
    ///
    /// SAFETY:
    /// - If the return value of `func` has to point to valid data of[`ComponentId`] type
    unsafe fn from_components_unchecked<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> *const u8) -> Self::Ref<'a>;

    /// Returns a tuple of mutable references to the components in the order of `Self`. The mutable
    /// references are set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components_mut<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::WrappedMutRef<'a>;

    /// Returns a tuple of mutable references to the components in the order of `Self`. The mutable
    /// references are set using the return value of `func`.
    ///
    /// SAFETY:
    /// - If the return value of `func` is has to point to valid data of[`ComponentId`] type
    unsafe fn from_components_unchecked_mut<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> *mut u8) -> Self::MutRef<'a>;

    /// Get the components of this [`Bundle`] with a corresponding [`ComponentId`]. This passes
    /// ownership to `func`.
    ///
    /// SAFETY:
    /// - pointer in `func` must be used, else will create memory leak if data has to be dropped
    /// - data in `func` must be manually dropped
    unsafe fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8));
}

unsafe impl<C: Component> Bundle for C {
    type Ref<'a> = &'a Self;
    type MutRef<'a> = &'a mut Self;

    type WrappedRef<'a> = Option<Self::Ref<'a>>;
    type WrappedMutRef<'a> = Option<Self::MutRef<'a>>;

    #[inline]
    fn empty_ref<'a>() -> Self::WrappedRef<'a> {
        None
    }

    #[inline]
    fn empty_mut_ref<'a>() -> Self::WrappedMutRef<'a> {
        None
    }

    #[inline]
    fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId)) {
        func(components.init::<C>())
    }

    #[inline]
    unsafe fn from_components<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::WrappedRef<'a> {
        Some(&*func(*components.get_id_from::<C>()?)?.cast::<Self>())
    }

    #[inline]
    unsafe fn from_components_unchecked<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> *const u8) -> Self::Ref<'a> {
        &*func(*components.get_id_from::<C>().unwrap()).cast::<Self>()
    }

    #[inline]
    unsafe fn from_components_mut<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::WrappedMutRef<'a> {
        Some(&mut *func(*components.get_id_from::<C>()?)?.cast::<Self>())
    }

    #[inline]
    unsafe fn from_components_unchecked_mut<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> *mut u8) -> Self::MutRef<'a> {
        &mut *func(*components.get_id_from::<C>().unwrap()).cast::<Self>()
    }

    #[inline]
    unsafe fn get_components(self, components: &Components, func: &mut impl FnMut(ComponentId, *mut u8)) {
        func(*components.get_id_from::<C>().unwrap(), &mut ManuallyDrop::new(self) as *mut ManuallyDrop<Self> as *mut u8)
    }
}

macro_rules! component_tuple_impl {
    ($($c:ident),*) => {
        unsafe impl<$($c: Bundle),*> Bundle for ($($c,)*) {
            type Ref<'a> = ($($c::Ref<'a>,)*);
            type MutRef<'a> = ($($c::MutRef<'a>,)*);

            type WrappedRef<'a> = ($($c::WrappedRef<'a>,)*);
            type WrappedMutRef<'a> = ($($c::WrappedMutRef<'a>,)*);

            #[inline]
            fn empty_ref<'a>() -> Self::WrappedRef<'a> {
                ($($c::empty_ref(),)*)
            }

            #[inline]
            fn empty_mut_ref<'a>() -> Self::WrappedMutRef<'a> {
                ($($c::empty_mut_ref(),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            fn component_ids(components: &mut Components, func: &mut impl FnMut(ComponentId)) {
                $(<$c as Bundle>::component_ids(components, func);)*
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::WrappedRef<'a> {
                ($($c::from_components(components, func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_unchecked<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> *const u8) -> Self::Ref<'a> {
                ($($c::from_components_unchecked(components, func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_mut<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::WrappedMutRef<'a> {
                ($($c::from_components_mut(components, func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_unchecked_mut<'a>(components: &Components, func: &mut impl FnMut(ComponentId) -> *mut u8) -> Self::MutRef<'a> {
                ($($c::from_components_unchecked_mut(components, func),)*)
            }

            #[inline]
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


/// Stores a sorted [`Vec`] of [`ComponentId`]s, used in [`Archetypes`](crate::archetype::Archetypes)
/// to to easily identify an [`Archetype`](crate::archetype::Archetype).
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
    pub fn remove(&mut self, components: Vec<ComponentId>) {
        let diff = self.components.clone().into_iter().filter(|id| !components.contains(id)).collect();
        self.components = diff;
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


/// Manages every [`Bundle`] that gets used on a [`World`](crate::world::World).
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
    pub fn init<'a, T: Bundle>(&'a mut self, components: &mut Components) -> &'a BundleInfo {
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

