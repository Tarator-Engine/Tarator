use std::mem::size_of;

pub type ComponentId = crate::id::Id;
pub type ComponentSet = std::collections::HashSet<ComponentId>;

/// implement by using #[derive(Component)]
pub trait Component {
    fn id() -> ComponentId;
}

pub trait ComponentTuple<'a> {
    fn set() -> ComponentSet {
        ComponentSet::new()
    }
    fn sizes() -> &'a [usize];
}


impl Component for () {
    fn id() -> ComponentId { ComponentId::MAX }
}
impl<'a, C: Component> ComponentTuple<'a> for C {
    fn set() -> ComponentSet {
        let mut set = ComponentSet::new();
        set.insert(C::id());
        set
    }
    fn sizes() -> &'a [usize] { &[size_of::<C>()] }
}

impl<'a, C0: Component, C1: Component> ComponentTuple<'a> for (C0, C1) {
    fn set() -> ComponentSet {
        let mut set = ComponentSet::new();
        set.insert(C0::id());
        set.insert(C1::id());
        set
    }
    fn sizes() -> &'a [usize] { &[size_of::<C0>(), size_of::<C1>()] }
}

impl<'a, C0: Component, C1: Component, C2: Component> ComponentTuple<'a> for (C0, C1, C2) {
    fn set() -> ComponentSet {
        let mut set = ComponentSet::new();
        set.insert(C0::id());
        set.insert(C1::id());
        set.insert(C2::id());
        set
    }
    fn sizes() -> &'a [usize] { &[size_of::<C0>(), size_of::<C1>(), size_of::<C2>()] }
}

impl<'a, C0: Component, C1: Component, C2: Component, C3: Component> ComponentTuple<'a> for (C0, C1, C2, C3) {
    fn set() -> ComponentSet {
        let mut set = ComponentSet::new();
        set.insert(C0::id());
        set.insert(C1::id());
        set.insert(C2::id());
        set.insert(C3::id());
        set
    }
    fn sizes() -> &'a [usize] { &[size_of::<C0>(), size_of::<C1>(), size_of::<C2>(), size_of::<C3>()] }
}

