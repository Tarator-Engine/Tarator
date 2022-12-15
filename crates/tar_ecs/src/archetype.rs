use crate::{
    component::*,
    error::EcsError as Error,
    id::*,
    storage::Storage,
    entity::*
};


pub(crate) type ArchetypeId = Id;

struct Archetype {
    id: ArchetypeId,
    parents: Vec<ArchetypeId>,
    components: ComponentSet,
    data: Storage
}

impl Archetype {
    fn new<'a, C: ComponentTuple<'a>>(id: ArchetypeId) -> Self {
        let mut ssize = 0;
        for size in C::sizes() {
            ssize += *size
        }
        Self {
            id,
            components: C::set(),
            parents: Vec::new(),
            data: unsafe { Storage::new(ssize, 4) } // 4 stands for "having room for 4 ComponentTuples"
        }
    }

    fn set<'a, C: ComponentTuple<'a>>(&mut self, desc: &mut Description, data: C) -> Result<(), Error> {
        desc.index = self.data.len();
        desc.id = DescriptionId::new(self.id, desc.id.get_version());
        self.data.push()?;
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
        let set = C::set();
        for arch in &mut self.arch {
            if set == arch.components {
                return arch.set(desc, data);
            }
        }

        let id = self.arch.len();
        let mut arch = Archetype::new::<C>(id);
        arch.set(desc, data)?;
        self.arch.push(arch);

        Ok(())
    }
}

