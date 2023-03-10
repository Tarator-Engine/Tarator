use std::{any::TypeId, collections::HashMap, mem::ManuallyDrop};

use crate::{
    component::{Component, ComponentId, Components},
    store::sparse::SparseSetIndex,
};
use fxhash::FxBuildHasher;
use parking_lot::RwLock;
use tar_ecs_macros::foreach_tuple;

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
    fn init_component_ids(func: &mut impl FnMut(ComponentId));

    fn get_component_ids(func: &mut impl FnMut(ComponentId));

    /// Returns a tuple of references to the components in the order of `Self`. The references are
    /// set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components<'a>(
        func: &mut impl FnMut(ComponentId) -> Option<*const u8>,
    ) -> Self::WrappedRef<'a>;

    /// Returns a tuple of references to the components in the order of `Self`. The references are
    /// set using the return value of `func`.
    ///
    /// SAFETY:
    /// - If the return value of `func` has to point to valid data of[`ComponentId`] type
    unsafe fn from_components_unchecked<'a>(
        func: &mut impl FnMut(ComponentId) -> *const u8,
    ) -> Self::Ref<'a>;

    /// Returns a tuple of mutable references to the components in the order of `Self`. The mutable
    /// references are set using the return value of `func`.
    ///
    /// SAFETY:
    /// - Returning [`None`] from `func` is always safe
    /// - If the return value of `func` is [`Some`], the pointer has to point to valid data of
    /// [`ComponentId`] type
    unsafe fn from_components_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> Option<*mut u8>,
    ) -> Self::WrappedMutRef<'a>;

    /// Returns a tuple of mutable references to the components in the order of `Self`. The mutable
    /// references are set using the return value of `func`.
    ///
    /// SAFETY:
    /// - If the return value of `func` is has to point to valid data of[`ComponentId`] type
    unsafe fn from_components_unchecked_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> *mut u8,
    ) -> Self::MutRef<'a>;

    /// Get the components of this [`Bundle`] with a corresponding [`ComponentId`]. This passes
    /// ownership to `func`.
    ///
    /// SAFETY:
    /// - pointer in `func` must be used, else will create memory leak if data has to be dropped
    /// - data in `func` must be manually dropped
    unsafe fn get_components(self, func: &mut impl FnMut(ComponentId, *mut u8));
}

/// Helper-trait to clone [`Bundle`]s with ease, where every [`Component`] has [`Clone`]
/// implemented
pub trait CloneBundle: Bundle + Clone {
    fn clone<'a>(data: Self::Ref<'a>) -> Self;
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
    fn init_component_ids(func: &mut impl FnMut(ComponentId)) {
        func(Components::init::<C>())
    }

    #[inline]
    fn get_component_ids(func: &mut impl FnMut(ComponentId)) {
        func(Components::get_id_from::<C>().unwrap())
    }

    #[inline]
    unsafe fn from_components<'a>(
        func: &mut impl FnMut(ComponentId) -> Option<*const u8>,
    ) -> Self::WrappedRef<'a> {
        Some(&*func(Components::get_id_from::<C>().unwrap())?.cast::<Self>())
    }

    #[inline]
    unsafe fn from_components_unchecked<'a>(
        func: &mut impl FnMut(ComponentId) -> *const u8,
    ) -> Self::Ref<'a> {
        &*func(Components::get_id_from::<C>().unwrap()).cast::<Self>()
    }

    #[inline]
    unsafe fn from_components_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> Option<*mut u8>,
    ) -> Self::WrappedMutRef<'a> {
        Some(&mut *func(Components::get_id_from::<C>().unwrap())?.cast::<Self>())
    }

    #[inline]
    unsafe fn from_components_unchecked_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> *mut u8,
    ) -> Self::MutRef<'a> {
        &mut *func(Components::get_id_from::<C>().unwrap()).cast::<Self>()
    }

    #[inline]
    unsafe fn get_components(self, func: &mut impl FnMut(ComponentId, *mut u8)) {
        func(
            Components::get_id_from::<C>().unwrap(),
            &mut ManuallyDrop::new(self) as *mut ManuallyDrop<Self> as *mut u8,
        )
    }
}

impl<C: Component + Clone> CloneBundle for C {
    fn clone<'a>(data: Self::Ref<'a>) -> Self {
        data.clone()
    }
}

macro_rules! component_tuple_impl {
    ($(($c:ident, $n:expr)),*) => {
        unsafe impl<$($c: Component + Bundle),*> Bundle for ($($c,)*) {
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
            fn init_component_ids(func: &mut impl FnMut(ComponentId)) {
                $(<$c as Bundle>::init_component_ids(func);)*
            }

            #[inline]
            #[allow(unused_variables)]
            fn get_component_ids(func: &mut impl FnMut(ComponentId)) {
                $(func(Components::get_id_from::<$c>().unwrap());)*
            }


            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components<'a>(func: &mut impl FnMut(ComponentId) -> Option<*const u8>) -> Self::WrappedRef<'a> {
                ($($c::from_components(func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_unchecked<'a>(func: &mut impl FnMut(ComponentId) -> *const u8) -> Self::Ref<'a> {
                ($($c::from_components_unchecked(func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_mut<'a>(func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::WrappedMutRef<'a> {
                ($($c::from_components_mut(func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_unchecked_mut<'a>(func: &mut impl FnMut(ComponentId) -> *mut u8) -> Self::MutRef<'a> {
                ($($c::from_components_unchecked_mut(func),)*)
            }

            #[inline]
            #[allow(unused_variables, unused_mut)]
            unsafe fn get_components(self, func: &mut impl FnMut(ComponentId, *mut u8)) {
                #[allow(non_snake_case)]
                let ($(mut $c,)*) = self;
                $($c.get_components(func);)*
            }
        }

        impl<$($c: Component + Clone),*> CloneBundle for ($($c,)*) {
            fn clone<'a>(data: Self::Ref<'a>) -> Self {
                #[allow(non_snake_case)]
                let ($($c,)*) = data;
                ($($c.clone(),)*)
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
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> usize {
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
    components: Vec<ComponentId>,
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
        let diff = self
            .components
            .clone()
            .into_iter()
            .filter(|id| !components.contains(id))
            .collect();
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

static mut BUNDLES: RwLock<Option<Bundles>> = RwLock::new(None);

pub struct Bundles {
    bundles: Vec<BundleComponents>,
    ids: HashMap<TypeId, BundleId, FxBuildHasher>,
}

impl Bundles {
    pub unsafe fn new() {
        let mut this = BUNDLES.write();

        if this.is_some() {
            return;
        }

        *this = Some(Self {
            bundles: Vec::new(),
            ids: Default::default(),
        })
    }

    pub fn init<T: Bundle>() -> BundleId {
        let mut components = Vec::new();
        T::init_component_ids(&mut |id| components.push(id));
        let len = components.len();
        components.sort();
        components.dedup();
        assert!(
            len == components.len(),
            "Bundle with duplicate components detected!"
        );
        let components = components.into();

        let mut this = unsafe { BUNDLES.write() };
        let this = this.as_mut().unwrap();

        let mut index = 0;
        for bundle in &this.bundles {
            if bundle == &components {
                return BundleId::new(index);
            }

            index += 1;
        }

        let id = BundleId::new(index);

        this.bundles.push(components);
        this.ids.insert(TypeId::of::<T>(), id);

        id
    }

    pub fn get_bundle<T>(id: BundleId, func: impl FnOnce(&BundleComponents) -> T) -> T {
        let this = unsafe { BUNDLES.read() };
        let this = this.as_ref().unwrap();

        let bundle = this.bundles.get(id.index()).unwrap();
        func(bundle)
    }
}
