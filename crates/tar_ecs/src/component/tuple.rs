use super::*;

pub trait ComponentTuple {
    fn get_ids() -> Vec<ComponentId> {
        vec![]
    }
}

impl ComponentTuple for () {}
impl<C: Component> ComponentTuple for C {
    fn get_ids() -> Vec<ComponentId> {
        vec![C::id()]
    }
}
impl<C0: Component, C1: Component> ComponentTuple for (C0, C1) {
    fn get_ids() -> Vec<ComponentId> {
        vec![C0::id(), C1::id()]
    }
}
impl<C0: Component, C1: Component, C2: Component> ComponentTuple for (C0, C1, C2) {
    fn get_ids() -> Vec<ComponentId> {
        vec![C0::id(), C1::id(), C2::id()]
    }
}
impl<C0: Component, C1: Component, C2: Component, C3: Component> ComponentTuple for (C0, C1, C2, C3) {
    fn get_ids() -> Vec<ComponentId> {
        vec![C0::id(), C1::id(), C2::id(), C3::id()]
    }
}

