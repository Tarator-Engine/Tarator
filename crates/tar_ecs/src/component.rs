use std::{
    mem::size_of,
    collections::HashSet,
    ptr::addr_of
};

pub type ComponentId = usize;
pub type ComponentSet = HashSet<ComponentId>;

/// implement by using #[derive(Component)]
pub trait Component {
    fn id() -> ComponentId;
}


#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TupleUnit {
    pub(crate) id: ComponentId,
    pub(crate) index: usize,
    pub(crate) size: usize,
    pub(crate) data: *const u8
}

pub trait ComponentTuple<'a> {
    fn set() -> ComponentSet {
        ComponentSet::new()
    }
    fn units(&self) -> Vec<TupleUnit>;
    fn size() -> usize;
}


#[macro_use]
mod macros {
    macro_rules! component_tuple {
        () => {
            impl<'a> ComponentTuple<'a> for () {
                fn units(&self) -> Vec<TupleUnit> {
                    vec![]
                }
                fn size() -> usize { size_of::<()>() }
            }
        };
        ($($c:tt), *) => {
            impl<'a,$($c: Component,)*> ComponentTuple<'a> for ($($c,)*) {
                fn set() -> ComponentSet {
                    let mut set = ComponentSet::new();
                    $(set.insert($c::id());)*
                    set
                }
                fn units(&self) -> Vec<TupleUnit> {
                    let mut units = vec![$(
                        TupleUnit {
                            id: $c::id(),
                            index: 0,
                            size: size_of::<$c>(),
                            data: unsafe{(addr_of!(self) as *const u8).add(size_of::<$c>())}
                        },
                    )*];
                    units.sort_unstable_by(|a, b| b.id.cmp(&a.id));
                    units
                }
                fn size() -> usize { $(size_of::<$c>()+)*0 }
            }
        };
    }
}



component_tuple!();
component_tuple!(C);
component_tuple!(C0, C1);
component_tuple!(C0, C1, C2);
component_tuple!(C0, C1, C2, C3);
component_tuple!(C0, C1, C2, C3, C4);
component_tuple!(C0, C1, C2, C3, C4, C5);
component_tuple!(C0, C1, C2, C3, C4, C5, C6);
component_tuple!(C0, C1, C2, C3, C4, C5, C6, C7);
component_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8);

