use std::collections::{HashMap, HashSet};

use fxhash::FxBuildHasher;

use crate::{
    bundle::{Bundle, BundleId, BundleInfo, BundleNames},
    callback::{Callback, CallbackFunc, CallbackId, CallbackName},
    component::{Component, ComponentId, ComponentInfo, ComponentName},
    store::sparse::SparseSetIndex,
};

pub trait TypeInfo: Sized {
    unsafe fn init_component(&mut self, name: ComponentName, info: ComponentInfo) -> ComponentId;
    fn get_component_id(&self, name: ComponentName) -> Option<ComponentId>;
    fn get_component_info<T>(
        &self,
        component_id: ComponentId,
        func: impl FnOnce(&ComponentInfo) -> T,
    ) -> Option<T>;
    unsafe fn get_component_info_mut<T>(
        &mut self,
        component_id: ComponentId,
        func: impl FnOnce(&mut ComponentInfo) -> T,
    ) -> Option<T>;

    unsafe fn component_add_callback(
        &mut self,
        component_id: ComponentId,
        callback_id: CallbackId,
        func: CallbackFunc,
    );
    fn init_callback(&mut self, name: CallbackName) -> CallbackId;
    fn get_callback_id(&self, name: &'static str) -> Option<CallbackId>;

    fn get_infos(&self, bundle_id: BundleId, func: impl FnMut(ComponentId, &ComponentInfo));
    fn init_bundle(&mut self, names: BundleNames) -> BundleId;
    fn insert_bundle(&mut self, info: BundleInfo) -> BundleId;
    fn get_bundle_info<T>(&self, id: BundleId, func: impl FnOnce(&BundleInfo) -> T) -> Option<T>;
    fn get_bundle_id(&self, names: BundleNames) -> Option<BundleId>;

    #[inline]
    fn init_component_from<T: Component>(&mut self) -> ComponentId {
        self.get_component_id_from::<T>().unwrap_or_else(|| unsafe {
            self.init_component(T::NAME, ComponentInfo::new_from::<T>())
        })
    }

    #[inline]
    fn get_component_id_from<T: Component>(&self) -> Option<ComponentId> {
        self.get_component_id(T::NAME)
    }

    #[inline]
    fn component_add_callback_from<T: Callback<U>, U: Component>(&mut self) {
        unsafe fn callback<T: Callback<U>, U: Component>(callback: *mut u8, component: *mut u8) {
            (*callback.cast::<T>()).callback(&mut *component.cast::<U>())
        }

        let callback_id = self.init_callback_from::<T, _>();
        let component_id = self.init_component_from::<U>();
        unsafe { self.component_add_callback(component_id, callback_id, callback::<T, U>) }
    }

    #[inline]
    fn init_callback_from<T: Callback<U>, U: Component>(&mut self) -> CallbackId {
        self.init_callback(T::NAME)
    }

    #[inline]
    fn get_callback_id_from<T: Callback<U>, U: Component>(&self) -> Option<CallbackId> {
        self.get_callback_id(T::NAME)
    }

    #[inline]
    fn get_infos_from<T: Bundle>(&self, func: impl FnMut(ComponentId, &ComponentInfo)) {
        if let Some(id) = self.get_bundle_id_from::<T>() {
            self.get_infos(id, func)
        }
    }

    fn init_bundle_from<T: Bundle>(&mut self) -> BundleId {
        T::init_component_ids(self, &mut |_| ());
        self.init_bundle(T::NAMES)
    }

    #[inline]
    fn get_bundle_info_from<T: Bundle, U>(&self, func: impl FnOnce(&BundleInfo) -> T) -> Option<T> {
        self.get_bundle_info(self.get_bundle_id_from::<T>()?, func)
    }

    #[inline]
    fn get_bundle_id_from<T: Bundle>(&self) -> Option<BundleId> {
        self.get_bundle_id(T::NAMES)
    }
}

pub struct Local {
    bundles: Vec<BundleInfo>,
    bundles_unsorted: HashMap<BundleNames, BundleId, FxBuildHasher>,
    components: Vec<ComponentInfo>,
    component_ids: HashMap<ComponentName, ComponentId, FxBuildHasher>,
    callback_ids: HashMap<CallbackName, CallbackId, FxBuildHasher>,
}

impl Local {
    #[inline]
    pub fn new() -> Self {
        Self {
            bundles: Vec::new(),
            bundles_unsorted: HashMap::default(),
            components: Vec::new(),
            component_ids: HashMap::default(),
            callback_ids: HashMap::default(),
        }
    }
}

impl TypeInfo for Local {
    #[inline]
    unsafe fn init_component(&mut self, name: ComponentName, info: ComponentInfo) -> ComponentId {
        let index = self.components.len();
        self.components.push(info);
        let id = ComponentId::from_usize(index);
        self.component_ids.insert(name, id);

        id
    }

    #[inline]
    fn get_component_id(&self, name: ComponentName) -> Option<ComponentId> {
        self.component_ids.get(&name).map(|id| *id)
    }

    #[inline]
    fn get_component_info<T>(
        &self,
        component_id: ComponentId,
        func: impl FnOnce(&ComponentInfo) -> T,
    ) -> Option<T> {
        Some(func(self.components.get(component_id.as_usize())?))
    }

    #[inline]
    unsafe fn get_component_info_mut<T>(
        &mut self,
        component_id: ComponentId,
        func: impl FnOnce(&mut ComponentInfo) -> T,
    ) -> Option<T> {
        Some(func(self.components.get_mut(component_id.as_usize())?))
    }

    #[inline]
    unsafe fn component_add_callback(
        &mut self,
        component_id: ComponentId,
        callback_id: CallbackId,
        func: CallbackFunc,
    ) {
        self.get_component_info_mut(component_id, |info| unsafe {
            info.set_callback(callback_id, func)
        })
        .expect("Callback wasn't initialized!")
    }

    #[inline]
    fn init_callback(&mut self, name: CallbackName) -> CallbackId {
        self.callback_ids
            .get(name)
            .map(|id| *id)
            .unwrap_or_else(|| CallbackId::from_usize(self.callback_ids.len()))
    }

    #[inline]
    fn get_callback_id(&self, name: CallbackName) -> Option<CallbackId> {
        self.callback_ids.get(name).map(|id| *id)
    }

    #[inline]
    fn get_infos(&self, bundle_id: BundleId, mut func: impl FnMut(ComponentId, &ComponentInfo)) {
        let Some(bundle_info) = self.bundles.get(bundle_id.as_usize()) else {
            return;
        };

        for id in bundle_info.component_ids() {
            if let Some(info) = self.components.get(id.as_usize()) {
                func(*id, info);
            }
        }
    }

    #[inline]
    fn init_bundle(&mut self, names: BundleNames) -> BundleId {
        let mut set = HashSet::default();

        for name in names {
            let id = self
                .get_component_id(name)
                .expect("Component not initialized!");
            set.insert(id);
        }

        let bundle_info = BundleInfo::new(set);

        for (i, bi) in self.bundles.iter().enumerate() {
            if bi == &bundle_info {
                let id = BundleId::from_usize(i);
                self.bundles_unsorted.insert(names, id);
                return id;
            }
        }

        let index = self.bundles.len();
        self.bundles.push(bundle_info);
        let id = BundleId::from_usize(index);
        self.bundles_unsorted.insert(names, id);

        id
    }

    #[inline]
    fn insert_bundle(&mut self, info: BundleInfo) -> BundleId {
        for (i, i_info) in self.bundles.iter().enumerate() {
            if *i_info == info {
                return BundleId::from_usize(i);
            }
        }

        let id = BundleId::from_usize(self.bundles.len());
        self.bundles.push(info);

        id
    }

    #[inline]
    fn get_bundle_info<T>(&self, id: BundleId, func: impl FnOnce(&BundleInfo) -> T) -> Option<T> {
        Some(func(self.bundles.get(id.as_usize())?))
    }

    #[inline]
    fn get_bundle_id(&self, names: BundleNames) -> Option<BundleId> {
        self.bundles_unsorted.get(names).map(|id| *id)
    }
}
