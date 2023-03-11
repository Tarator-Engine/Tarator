use std::{any::type_name, collections::HashMap, mem::ManuallyDrop};

use crate::{
    component::{Component, ComponentHashId, ComponentId, Components},
    store::sparse::SparseSetIndex,
};
use fxhash::{hash, FxBuildHasher};
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
/// - [`Bundle::WrappedRef`] and [`Bundle::WrappedMut`] are supposed to be wrapped in[`Option`]
pub unsafe trait Bundle: Send + Sync + 'static {
    /// Implemented as a tuple of [`Component`] references
    type Ref<'a>: Copy;

    /// Implemented as a tuple of mutable [`Component`] references
    type Mut<'a>;

    /// Implemented as a tuple of mutable [`Component`] raw pointer
    type RawMut: Copy;

    /// Implemented as a tuple of [`Component`] references wrapped in [`Option`]
    type WrappedRef<'a>;

    /// Implemented as a tuple of mutable [`Component`] references wrapped in [`Option`]
    type WrappedMut<'a>;

    /// Returns a tuple of [`None`] values, used to return from a function, if for example an
    /// [`Entity`](crate::entity::Entity) doesn't exist.
    fn empty_ref<'a>() -> Self::WrappedRef<'a>;

    /// Returns a tuple of [`None`] values, used to return from a function, if for example an
    /// [`Entity`](crate::entity::Entity) doesn't exist.
    fn empty_mut<'a>() -> Self::WrappedMut<'a>;

    /// Turns [`Bundle::RawMut`] into [`Bundle::Ref`]
    unsafe fn into_ref<'a>(data: Self::RawMut) -> Self::Ref<'a>;

    /// Turns [`Bundle::RawMut`] into [`Bundle::Mut`]
    unsafe fn into_mut<'a>(data: Self::RawMut) -> Self::Mut<'a>;

    /// Turn [`Bundle::WrappedRef`] into [`Some(_)`] if all [`Bundle::WrappedRef`]s are [`Some(_)`] or into [`None`]
    fn some_ref_or_none<'a>(data: Self::WrappedRef<'a>) -> Option<Self::Ref<'a>>;

    /// Turn [`Bundle::WrappedMut`] into [`Some(_)`] if all [`Bundle::WrappedMut`]s are [`Some(_)`] or into [`None`]
    fn some_mut_or_none<'a>(data: Self::WrappedMut<'a>) -> Option<Self::Mut<'a>>;

    /// Turn [`Bundle::WrappedMut`] into [`Some(_)`] if all [`Bundle::WrappedMut`]s are [`Some(_)`] or into [`None`]
    fn some_raw_mut_or_none<'a>(data: Self::WrappedMut<'a>) -> Option<Self::RawMut>;

    /// Initializes and gets the [`ComponentId`]s via `func`
    fn init_component_ids(func: &mut impl FnMut(ComponentId));

    /// Gets the [`ComponentId`]s via `func`
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
    ) -> Self::WrappedMut<'a>;

    /// Returns a tuple of mutable references to the components in the order of `Self`. The mutable
    /// references are set using the return value of `func`.
    ///
    /// SAFETY:
    /// - If the return value of `func` is has to point to valid data of[`ComponentId`] type
    unsafe fn from_components_unchecked_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> *mut u8,
    ) -> Self::Mut<'a>;

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

unsafe impl<T: Component> Bundle for T {
    type Ref<'a> = &'a Self;
    type Mut<'a> = &'a mut Self;
    type RawMut = *mut Self;

    type WrappedRef<'a> = Option<Self::Ref<'a>>;
    type WrappedMut<'a> = Option<Self::Mut<'a>>;

    #[inline]
    fn empty_ref<'a>() -> Self::WrappedRef<'a> {
        None
    }

    #[inline]
    fn empty_mut<'a>() -> Self::WrappedMut<'a> {
        None
    }

    #[inline]
    unsafe fn into_ref<'a>(data: Self::RawMut) -> Self::Ref<'a> {
        &*data
    }

    #[inline]
    unsafe fn into_mut<'a>(data: Self::RawMut) -> Self::Mut<'a> {
        &mut *data
    }

    #[inline]
    fn some_ref_or_none<'a>(data: Self::WrappedRef<'a>) -> Option<Self::Ref<'a>> {
        data
    }

    #[inline]
    fn some_mut_or_none<'a>(data: Self::WrappedMut<'a>) -> Option<Self::Mut<'a>> {
        data
    }

    #[inline]
    fn some_raw_mut_or_none<'a>(data: Self::WrappedMut<'a>) -> Option<Self::RawMut> {
        Some(data? as *mut _)
    }

    #[inline]
    fn init_component_ids(func: &mut impl FnMut(ComponentId)) {
        func(Components::init::<T>())
    }

    #[inline]
    fn get_component_ids(func: &mut impl FnMut(ComponentId)) {
        func(Components::get_id_from::<T>().unwrap())
    }

    #[inline]
    unsafe fn from_components<'a>(
        func: &mut impl FnMut(ComponentId) -> Option<*const u8>,
    ) -> Self::WrappedRef<'a> {
        Some(&*func(Components::get_id_from::<T>().unwrap())?.cast::<Self>())
    }

    #[inline]
    unsafe fn from_components_unchecked<'a>(
        func: &mut impl FnMut(ComponentId) -> *const u8,
    ) -> Self::Ref<'a> {
        &*func(Components::get_id_from::<T>().unwrap()).cast::<Self>()
    }

    #[inline]
    unsafe fn from_components_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> Option<*mut u8>,
    ) -> Self::WrappedMut<'a> {
        Some(&mut *func(Components::get_id_from::<T>().unwrap())?.cast::<Self>())
    }

    #[inline]
    unsafe fn from_components_unchecked_mut<'a>(
        func: &mut impl FnMut(ComponentId) -> *mut u8,
    ) -> Self::Mut<'a> {
        &mut *func(Components::get_id_from::<T>().unwrap()).cast::<Self>()
    }

    #[inline]
    unsafe fn get_components(self, func: &mut impl FnMut(ComponentId, *mut u8)) {
        func(
            Components::get_id_from::<T>().unwrap(),
            &mut ManuallyDrop::new(self) as *mut ManuallyDrop<Self> as *mut u8,
        )
    }
}

impl<T: Component + Clone> CloneBundle for T {
    fn clone<'a>(data: Self::Ref<'a>) -> Self {
        data.clone()
    }
}

macro_rules! component_tuple_impl {
    ($($c:ident),*) => {
        unsafe impl<$($c: Component + Bundle),*> Bundle for ($($c,)*) {
            type Ref<'a> = ($($c::Ref<'a>,)*);
            type Mut<'a> = ($($c::Mut<'a>,)*);
            type RawMut = ($($c::RawMut,)*);

            type WrappedRef<'a> = ($($c::WrappedRef<'a>,)*);
            type WrappedMut<'a> = ($($c::WrappedMut<'a>,)*);

            #[inline]
            fn empty_ref<'a>() -> Self::WrappedRef<'a> {
                ($($c::empty_ref(),)*)
            }

            #[inline]
            fn empty_mut<'a>() -> Self::WrappedMut<'a> {
                ($($c::empty_mut(),)*)
            }

            #[inline]
            unsafe fn into_ref<'a>(data: Self::RawMut) -> Self::Ref<'a> {
                #[allow(non_snake_case)]
                let ($($c,)*) = data;
                ($($c::into_ref($c),)*)
            }

            #[inline]
            unsafe fn into_mut<'a>(data: Self::RawMut) -> Self::Mut<'a> {
                #[allow(non_snake_case)]
                let ($($c,)*) = data;
                ($($c::into_mut($c),)*)
            }

            #[inline]
            fn some_ref_or_none<'a>(data: Self::WrappedRef<'a>) -> Option<Self::Ref<'a>> {
                #[allow(non_snake_case)]
                let ($($c,)*) = data;
                Some(($( $c::some_ref_or_none($c)?, )*))
            }

            #[inline]
            fn some_mut_or_none<'a>(data: Self::WrappedMut<'a>) -> Option<Self::Mut<'a>> {
                #[allow(non_snake_case)]
                let ($($c,)*) = data;
                Some(($( $c::some_mut_or_none($c)?, )*))
            }

            #[inline]
            fn some_raw_mut_or_none<'a>(data: Self::WrappedMut<'a>) -> Option<Self::RawMut> {
                #[allow(non_snake_case)]
                let ($($c,)*) = data;
                Some(($( $c::some_raw_mut_or_none($c)?, )*))
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
            unsafe fn from_components_mut<'a>(func: &mut impl FnMut(ComponentId) -> Option<*mut u8>) -> Self::WrappedMut<'a> {
                ($($c::from_components_mut(func),)*)
            }

            #[inline]
            #[allow(unused_variables)]
            unsafe fn from_components_unchecked_mut<'a>(func: &mut impl FnMut(ComponentId) -> *mut u8) -> Self::Mut<'a> {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BundleHashId(usize);

impl BundleHashId {
    #[inline]
    pub fn new<T: Bundle>() -> Self {
        Self::new_from_str(type_name::<T>())
    }

    #[inline]
    pub fn new_from_str(name: &'static str) -> Self {
        Self(hash(name))
    }

    #[inline]
    pub const fn id(self) -> usize {
        self.0
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
    ids: HashMap<BundleHashId, BundleId, FxBuildHasher>,
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
        unsafe { Self::new() };

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
        this.ids.insert(BundleHashId::new::<T>(), id);

        id
    }

    pub fn init_from_name(name: &'static str) -> BundleId {
        unsafe { Self::new() };

        let mut this = unsafe { BUNDLES.write() };
        let this = this.as_mut().unwrap();

        this.ids
            .get(&BundleHashId::new_from_str(name))
            .map(|id| *id)
            .unwrap_or_else(|| {
                let names: Vec<_> = name
                    .strip_prefix("(")
                    .unwrap_or_else(|| name)
                    .strip_suffix(")")
                    .unwrap_or_else(|| name)
                    .split(", ")
                    .collect();

                let mut components = Vec::with_capacity(names.len());

                for n in names {
                    components.push(Components::get_id(ComponentHashId::new_from_str(n)).unwrap())
                }

                let len = components.len();
                components.sort();
                components.dedup();
                assert!(
                    len == components.len(),
                    "Bundle with duplicate components detected: ({})!",
                    name
                );

                let components = BundleComponents::new(components);

                let mut index = 0;
                for bundle in &this.bundles {
                    if bundle == &components {
                        let id = BundleId::new(index);
                        this.ids.insert(BundleHashId::new_from_str(name), id);

                        return id;
                    }

                    index += 1;
                }

                let id = BundleId::new(index);

                this.bundles.push(components);
                this.ids.insert(BundleHashId::new_from_str(name), id);

                id
            })
    }

    pub fn get_bundle<T>(id: BundleId, func: impl FnOnce(&BundleComponents) -> T) -> T {
        let this = unsafe { BUNDLES.read() };
        let this = this.as_ref().unwrap();

        let bundle = this.bundles.get(id.index()).unwrap();
        func(bundle)
    }

    pub fn get_bundle_id(hash_id: BundleHashId) -> Option<BundleId> {
        let this = unsafe { BUNDLES.read() };
        let this = this.as_ref().unwrap();

        this.ids.get(&hash_id).map(|id| *id)
    }
}
