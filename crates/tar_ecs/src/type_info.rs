use std::{collections::{HashMap, HashSet}, any::TypeId};

use fxhash::FxBuildHasher;

use crate::{
    bundle::{Bundle, BundleId, BundleInfo},
    callback::{Callback, CallbackId},
    component::{Component, ComponentId, ComponentInfo},
    store::sparse::SparseSetIndex,
};

pub trait TypeInfo: Sized {
    fn init_component_from<T: Component>(&mut self) -> ComponentId;
    fn get_component_id_from<T: Component>(&self) -> Option<ComponentId>;
    fn get_component_info<T>(&self, component_id: ComponentId, func: impl FnOnce(&ComponentInfo) -> T) -> Option<T>;
    fn get_component_info_mut<T>(&mut self, component_id: ComponentId, func: impl FnOnce(&mut ComponentInfo) -> T) -> Option<T>;

    fn get_infos(&self, bundle_id: BundleId, func: impl FnMut(ComponentId, &ComponentInfo));

    fn init_bundle_from<T: Bundle>(&mut self) -> BundleId;
    fn insert_bundle(&mut self, bundle_info: BundleInfo) -> BundleId;
    fn get_bundle_id_from<T: Bundle>(&self) -> Option<BundleId>;
    fn get_bundle_info<T>(&self, bundle_id: BundleId, func: impl FnOnce(&BundleInfo) -> T) -> Option<T>;

    fn init_callback_from<T: Callback<U>, U: Component>(&mut self) -> CallbackId;
    fn get_callback_id_from<T: Callback<U>, U: Component>(&self) -> Option<CallbackId>;
    fn component_add_callback_from<T: Callback<U>, U: Component>(&mut self);
}

pub struct Local {
    bundles: Vec<BundleInfo>,
    bundle_ids: HashMap<TypeId, BundleId, FxBuildHasher>,
    components: Vec<ComponentInfo>,
    component_ids: HashMap<TypeId, ComponentId, FxBuildHasher>,
    callback_ids: HashMap<TypeId, CallbackId, FxBuildHasher>,
}

impl Local {
    #[inline]
    pub fn new() -> Self {
        Self {
            bundles: Vec::new(),
            bundle_ids: HashMap::default(),
            components: Vec::new(),
            component_ids: HashMap::default(),
            callback_ids: HashMap::default(),
        }
    }
}

impl TypeInfo for Local {
    #[inline]
    fn init_component_from<T: Component>(&mut self) -> ComponentId {
        self.component_ids
            .get(&T::type_id())
            .map(|id| *id)
            .unwrap_or_else(|| {
                let index = self.components.len();
                self.components.push(ComponentInfo::new_from::<T>());
                let id = ComponentId::from_usize(index);
                self.component_ids.insert(T::type_id(), id);

                id
            })
    }

    #[inline]
    fn get_component_id_from<T: Component>(&self) -> Option<ComponentId> {
        self.component_ids.get(&T::type_id()).map(|id| *id)
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
    fn get_component_info_mut<T>(
        &mut self,
        component_id: ComponentId,
        func: impl FnOnce(&mut ComponentInfo) -> T,
    ) -> Option<T> {
        Some(func(self.components.get_mut(component_id.as_usize())?))
    }

    #[inline]
    fn component_add_callback_from<T: Callback<U>, U: Component>(&mut self) {
        let component_id = self.init_component_from::<U>();
        let callback_id = self.init_callback_from::<T, _>();

        self.get_component_info_mut(component_id, |info| {
            unsafe fn callback<T: Callback<U>, U: Component>(callback: *mut u8, component: *mut u8) {
                (*callback.cast::<T>()).callback(&mut *component.cast::<U>())
            }

            unsafe {
                info.set_callback(callback_id, callback::<T, U>)
            }
        }).unwrap()
    }

    #[inline]
    fn init_callback_from<T: Callback<U>, U: Component>(&mut self) -> CallbackId {
        self.get_callback_id_from::<T, _>().unwrap_or_else(|| {
            let id = CallbackId::from_usize(self.callback_ids.len());
            self.callback_ids.insert(T::type_id(), id);

            id
        })
    }

    #[inline]
    fn get_callback_id_from<T: Callback<U>, U: Component>(&self) -> Option<CallbackId> {
        self.callback_ids.get(&T::type_id()).map(|id| *id)
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
    fn init_bundle_from<T: Bundle>(&mut self) -> BundleId {
        self.get_bundle_id_from::<T>().unwrap_or_else(|| {
            let mut set = HashSet::default();
            T::init_component_ids(self, &mut |id| { set.insert(id); });

            let id = self.insert_bundle(BundleInfo::new(set));
            self.bundle_ids.insert(T::b_type_id(), id);

            id
        })
    }

    #[inline]
    fn insert_bundle(&mut self, info: BundleInfo) -> BundleId {
        for (i, i_info) in self.bundles.iter().enumerate() {
            if i_info == &info {
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
    fn get_bundle_id_from<T: Bundle>(&self) -> Option<BundleId> {
        self.bundle_ids.get(&T::b_type_id()).map(|id| *id)
    }
}
