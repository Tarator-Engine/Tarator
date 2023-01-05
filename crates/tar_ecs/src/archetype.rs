use crate::{
    component::*,
    error::EcsError as Error,
    id::*,
    storage::Storage,
    entity::*
};


pub(crate) type ArchetypeId = usize;

struct Archetype {
    id: ArchetypeId,
    parents: Vec<ArchetypeId>,
    set: ComponentSet,
    data: Storage
}

impl Archetype {
    fn new(id: ArchetypeId, set: ComponentSet, size: usize) -> Result<Self, Error> {
        Ok(Self {
            id, set,
            parents: Vec::new(),
            data: unsafe { Storage::new(size, 16)? } // 4 stands for "having room for 4 ComponentTuples"
        })
    }

    fn set(&mut self, desc: &mut Description, unit: TupleUnit) -> Result<(), Error> {
        if !desc.is_index_valid() {
            let index = self.data.len();
            self.data.increase()?;
            unsafe { self.data.set(index, unit)?; }

            desc.index = index;
            desc.id = DescriptionId::new(self.id, desc.id.get_version());
        } else {
            todo!()
        }

        Ok(())
    }
}


pub(crate) struct ArchetypePool {
    arch: Vec<Archetype>
}

impl ArchetypePool {
    pub(crate) fn new() -> Self {
        Self {
            arch: Vec::new()
        }
    }
    pub(crate) fn set<'a, C: ComponentTuple<'a>>(&mut self, desc: &mut Description, data: C) -> Result<(), Error> {
        let archetype: &mut Archetype;
        'getter: {
            let set = C::set();
            for arch in &mut self.arch {
                if set == arch.set {
                    archetype = arch;
                    break 'getter;
                }
            }
            let id = self.arch.len();
            let arch = Archetype::new(id, set, C::size())?;
            self.arch.push(arch);
            archetype = unsafe{self.arch.get_unchecked_mut(id)};
        }

        let mut index = 0;
        for mut unit in data.units() {
            unit.index = index;
            index += unit.size;
            archetype.set(desc, unit)?;
        }

        Ok(())
    }
}

