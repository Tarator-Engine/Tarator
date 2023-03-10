use std::{
    alloc::Layout,
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    mem::{self, needs_drop, size_of},
    ptr::{addr_of_mut, copy, drop_in_place},
    sync::Arc,
};

use fxhash::FxBuildHasher;
use parking_lot::{Mutex, MutexGuard, RwLock};
use tar_ecs_macros::Component;

use crate::{
    archetype::{ArchetypeId, Archetypes},
    bundle::Bundle,
    callback::{Callback, CallbackFunc, CallbackId, ComponentCallbacks},
    store::{sparse::SparseSetIndex, table::Table},
};

/// A [`Component`] is nothing more but data, which can be stored in a given
/// [`World`](crate::world::World) on an [`Entity`](crate::entity::Entity). [`Component`] can
/// be derived via `#[derive(Component)]`.
///
/// Read further: [`Bundle`]
///
/// SAFETY:
/// - Manual implementation is discouraged
pub unsafe trait Component: Sized + Send + Sync + 'static {
    #[inline]
    fn add_callback<T: Callback<Self>>() {
        unsafe {
            Components::new();
        }
        Components::add_callback::<T, Self>()
    }
}

#[derive(Component)]
pub struct Fake;

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

pub struct ComponentInfo {
    drop: Option<unsafe fn(*mut u8)>,
    layout: Layout,
    callbacks: ComponentCallbacks,
}

impl ComponentInfo {
    unsafe fn inner_drop<T>(to_drop: *mut u8) {
        to_drop.cast::<T>().drop_in_place()
    }

    #[inline]
    pub fn new_from<T: Component>() -> Self {
        Self {
            drop: needs_drop::<T>().then_some(Self::inner_drop::<T>),
            layout: Layout::new::<T>(),
            callbacks: ComponentCallbacks::new(),
        }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.layout
    }

    #[inline]
    pub fn drop(&self) -> Option<unsafe fn(*mut u8)> {
        self.drop
    }

    #[inline]
    pub fn callback(&self, id: CallbackId) -> Option<CallbackFunc> {
        self.callbacks.get(id)
    }
}

static mut COMPONENTS: RwLock<Option<Components>> = RwLock::new(None);

pub struct Components {
    infos: Vec<ComponentInfo>,
    ids: HashMap<TypeId, ComponentId, FxBuildHasher>,
    callback_ids: HashMap<TypeId, CallbackId, FxBuildHasher>,
}

impl Components {
    pub unsafe fn new() {
        let mut this = COMPONENTS.write();

        if this.is_some() {
            return;
        }

        *this = Some(Self {
            infos: Default::default(),
            ids: Default::default(),
            callback_ids: Default::default(),
        })
    }

    pub fn init<T: Component>() -> ComponentId {
        let mut this = unsafe { COMPONENTS.write() };
        let this = this.as_mut().unwrap();

        this.ids
            .get(&TypeId::of::<T>())
            .map(|id| *id)
            .unwrap_or_else(|| {
                let index = this.infos.len();
                let id = ComponentId::new(index);
                this.ids.insert(TypeId::of::<T>(), id);
                this.infos.push(ComponentInfo::new_from::<T>());

                id
            })
    }

    pub fn add_callback<T: Callback<U>, U: Component>() {
        let mut this = unsafe { COMPONENTS.write() };
        let this = this.as_mut().unwrap();

        let callback_type_id = TypeId::of::<T>();
        let callback_id = this
            .callback_ids
            .get(&callback_type_id)
            .map(|id| *id)
            .unwrap_or_else(|| {
                let index = this.callback_ids.len();
                let callback_id = CallbackId::new(index);
                this.callback_ids.insert(callback_type_id, callback_id);

                callback_id
            });

        let id = this
            .ids
            .get(&TypeId::of::<U>())
            .map(|id| *id)
            .unwrap_or_else(|| {
                let index = this.infos.len();
                let id = ComponentId::new(index);
                this.ids.insert(TypeId::of::<U>(), id);
                this.infos.push(ComponentInfo::new_from::<U>());

                id
            });
        // SAFETY:
        // We just checked or pushed
        let info = unsafe { this.infos.get_unchecked_mut(id.index()) };

        info.callbacks.add::<T, U>(callback_id);
    }

    pub fn get_info<T>(id: ComponentId, func: impl FnOnce(&ComponentInfo) -> T) -> T {
        let this = unsafe { COMPONENTS.read() };
        let this = this.as_ref().unwrap();
        let info = this.infos.get(id.index()).unwrap();

        func(info)
    }

    pub fn get_id_from<T: Component>() -> Option<ComponentId> {
        Self::get_id(TypeId::of::<T>())
    }

    pub fn get_id(id: TypeId) -> Option<ComponentId> {
        let this = unsafe { COMPONENTS.read() };
        let this = this.as_ref().unwrap();
        this.ids.get(&id).map(|id| *id)
    }

    pub fn get_callback_id_from<T: Callback<Fake>>() -> Option<CallbackId> {
        Self::get_callback_id(TypeId::of::<T>())
    }

    pub fn get_callback_id(id: TypeId) -> Option<CallbackId> {
        let this = unsafe { COMPONENTS.read() };
        let this = this.as_ref().unwrap();
        this.callback_ids.get(&id).map(|id| *id)
    }
}

/// An [`Iterator`] for a given [`Bundle`], which iterates over all
/// [`Archetype`](crate::archetype::Archetype)s of a [`World`](crate::world::World) who contain the
/// [`Bundle`].
#[derive(Debug)]
pub struct ComponentQuery<'a, T: Bundle> {
    tables: Vec<Arc<Mutex<Table>>>,
    current: MutexGuard<'a, Table>,
    index: usize,
    table: usize,
    marker: PhantomData<&'a T>,
}

impl<'a, T: Bundle> ComponentQuery<'a, T> {
    pub fn new(archetype_ids: &Vec<ArchetypeId>, archetypes: &'a mut Archetypes) -> Self {
        let mut tables = Vec::with_capacity(archetype_ids.len());

        for id in archetype_ids {
            if let Some(archetype) = archetypes.get(*id) {
                tables.push(archetype.table())
            } else {
                debug_assert!(false, "Invalid Id was passed!");
            }
        }

        Self {
            current: archetypes.get(archetype_ids[0]).unwrap().table_lock(),
            tables,
            index: 0,
            table: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Bundle> Iterator for ComponentQuery<'a, T> {
    type Item = T::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if self.current.len() > index {
            self.index += 1;
            // SAFETY:
            // Just boundchecked
            return unsafe { Some(self.current.get_unchecked::<T>(index)) };
        }

        self.index = 0;
        self.table += 1;

        // This here is some magic to keep the Mutex locked for it's entire iteration
        unsafe {
            let mut table = self.tables.get(self.table)?.lock();
            let table = addr_of_mut!(table);
            let current = addr_of_mut!(self.current).clone();

            // Drop the current Guard, unlocking the mutex
            drop_in_place(current);

            // Copy in the new guard
            copy(
                table.cast::<u8>(),
                current.cast::<u8>(),
                size_of::<MutexGuard<'a, Table>>(),
            );

            // Forget the local variable of the lock, so that our mutex doesn't get replaced
            mem::forget(table);
        }

        return self.next();
    }
}

/// An [`Iterator`] for a given [`Bundle`], which iterates mutably over all
/// [`Archetype`](crate::archetype::Archetype)s of a [`World`](crate::world::World) who contain the
/// [`Bundle`].
#[derive(Debug)]
pub struct ComponentQueryMut<'a, T: Bundle> {
    tables: Vec<Arc<Mutex<Table>>>,
    current: MutexGuard<'a, Table>,
    index: usize,
    table: usize,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T: Bundle> ComponentQueryMut<'a, T> {
    pub fn new(archetype_ids: &Vec<ArchetypeId>, archetypes: &'a mut Archetypes) -> Self {
        let mut tables = Vec::with_capacity(archetype_ids.len());

        for id in archetype_ids {
            if let Some(archetype) = archetypes.get(*id) {
                tables.push(archetype.table())
            } else {
                debug_assert!(false, "Invalid Id was passed!");
            }
        }

        Self {
            tables,
            current: archetypes.get(archetype_ids[0]).unwrap().table_lock(),
            index: 0,
            table: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: Bundle> Iterator for ComponentQueryMut<'a, T> {
    type Item = T::Mut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if self.current.len() > index {
            self.index += 1;
            return unsafe { Some(self.current.get_unchecked_mut::<T>(index)) };
        }

        self.index = 0;
        self.table += 1;

        // This here is some magic to keep the Mutex locked for it's entire iteration
        unsafe {
            let mut table = self.tables.get(self.table)?.lock();
            let table = addr_of_mut!(table);
            let current = addr_of_mut!(self.current).clone();

            // Drop the current Guard, unlocking the mutex
            drop_in_place(current);

            // Copy in the new guard
            copy(
                table.cast::<u8>(),
                current.cast::<u8>(),
                size_of::<MutexGuard<'a, Table>>(),
            );

            // Forget the local variable of the lock, so that our mutex doesn't get replaced
            mem::forget(table);
        }

        return self.next();
    }
}
