use std::{
    mem::size_of,
    ptr::addr_of, sync::atomic::AtomicBool
};
use super::*;


pub trait Unit {
    fn size(&self) -> usize;
    fn set_offset(&mut self, offset: usize);
}

#[derive(Clone, Copy)]
pub struct TupleUnit {
    pub(crate) id: ComponentId,
    pub(crate) offset: usize,
}

impl Unit for TupleUnit {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn set_offset(&mut self, offset: usize) {
        self.offset = offset
    }
}


#[derive(Clone, Copy)]
pub struct DataUnit {
    pub(crate) id: ComponentId,
    pub(crate) offset: usize,
    pub(crate) data: *const u8
}

impl Unit for DataUnit {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn set_offset(&mut self, offset: usize) {
        self.offset = offset
    }
}

pub struct UnitIter<T: Unit> {
    current: usize,
    offset: usize,
    units: Vec<T>
}

impl<T: Unit> UnitIter<T> {
    fn new(units: Vec<T>) -> Self {
        Self {
            current: 0,
            offset: 0,
            units
        }
    }
}

impl<T: Unit + Copy> Iterator for UnitIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let ret = self.units.get_mut(self.current)?;
        ret.set_offset(self.offset);
        self.offset = ret.size() + size_of::<AtomicBool>();
        self.current += 1;

        Some(*ret)
    }
}



pub trait ComponentTuple {
    fn set() -> ComponentSet {
        ComponentSet::new()
    }
    fn tuple_units() -> UnitIter<TupleUnit>;
    fn data_units(&self) -> UnitIter<DataUnit>;
    fn size() -> usize;
}


#[macro_use]
mod macros {
    macro_rules! component_tuple {
        () => {
            impl ComponentTuple for () {
                fn data_units(&self) -> UnitIter<DataUnit> {
                    UnitIter::new(vec![])
                }
                fn tuple_units() -> UnitIter<TupleUnit> {
                    UnitIter::new(vec![])
                }
                fn size() -> usize { size_of::<()>() }
            }
        };
        ($($c:tt), *) => {
            impl<$($c: Component,)*> ComponentTuple for ($($c,)*) {
                fn set() -> ComponentSet {
                    let mut set = ComponentSet::new();
                    $(set.insert($c::id());)*
                    set
                }
                // TODO code seems inefficient here...
                fn tuple_units() -> UnitIter<TupleUnit> {
                    let mut units = vec![$(
                        TupleUnit {
                            id: $c::id(),
                            offset: 0,
                        },
                    )*];
                    units.sort_unstable_by(|a, b| b.id.cmp(&a.id));

                    UnitIter::new(units)
                }
                // TODO code seems inefficient here...
                fn data_units(&self) -> UnitIter<DataUnit> {
                    let mut units = vec![$(
                        DataUnit {
                            id: $c::id(),
                            offset: 0,
                            data: unsafe{(addr_of!(self) as *const u8).add(size_of::<$c>())}
                        },
                    )*];
                    units.sort_unstable_by(|a, b| b.id.cmp(&a.id));

                    UnitIter::new(units)
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

