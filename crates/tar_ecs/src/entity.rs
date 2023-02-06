use crate::archetype::ArchetypeId;


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Entity(u32, u32);

impl Entity {
    pub const INVALID: Self = Self(u32::MAX, u32::MAX);

    #[inline]
    pub fn new(id: u32, version: u32) -> Self {
        Self(id, version)
    }

    #[inline]
    pub fn id(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn version(self) -> u32 {
        self.1
    }
}


#[derive(Debug)]
pub struct EntityMeta {
    pub archetype_id: ArchetypeId,
    pub index: usize,
    pub version: u32
}


/// Stores all the data information of our entities and checks if they are still alive.
///
/// The implementation is based on [this blog post](https://skypjack.github.io/2019-05-06-ecs-baf-part-3)
#[derive(Debug)]
pub struct Entities {
    meta: Vec<EntityMeta>,
    free_count: usize,
    free_next: usize
}

impl Entities {
    #[inline]
    pub fn new() -> Self {
        Self {
            meta: Vec::new(),
            free_count: 0,
            free_next: 0
        }
    }

    pub fn create(&mut self) -> Entity {
        if self.free_count == 0 {
            let index = self.meta.len() as u32;
            self.meta.push(EntityMeta {
                archetype_id: ArchetypeId::EMPTY,
                version: 0,
                index: 0
            });

            return Entity::new(index, 0);
        }

        let id = self.free_next as u32;

        let free = &mut self.meta[self.free_next];
        // Free entities should have an [`ArchetypeId`] variant [`ArchetypeId::INVALID`]
        debug_assert!(free.archetype_id == ArchetypeId::INVALID);

        // set `free_next` to the index our free had pointed to
        self.free_next = free.index;
        self.free_count -= 1;

        // Set our freed [`EntityMeta`]
        free.archetype_id = ArchetypeId::EMPTY;
        free.version += 1;
        free.index = 0;

        Entity::new(id, free.version)
    }

    /// Returns None if the entity was already destroyed or reoccupied
    pub fn destroy(&mut self, entity: Entity) -> Option<EntityMeta> {
        let index = entity.id() as usize;
        let meta = &mut self.meta[index];

        // Ignore if:
        // - Version differs
        // - Entity is already destroyed
        if meta.version != entity.version() || meta.archetype_id == ArchetypeId::INVALID {
            return None;
        }

        let old_meta = EntityMeta {
            archetype_id: meta.archetype_id,
            version: meta.version,
            index: meta.index
        };

        // Set the index of our `EntityMeta` that we want to destory to the current `free_next`,
        // and set `free_next` to our index, as well as increment `free_count`
        meta.index = self.free_next;
        self.free_next = index;
        self.free_count += 1;

        meta.archetype_id = ArchetypeId::INVALID;

        Some(old_meta)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&EntityMeta> {
        let meta = &self.meta[entity.id() as usize];

        if meta.version != entity.version() || meta.archetype_id == ArchetypeId::INVALID {
            return None;
        }

        Some(meta)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityMeta> {
        let meta = &mut self.meta[entity.id() as usize];

        if meta.version != entity.version() || meta.archetype_id == ArchetypeId::INVALID {
            return None;
        }

        Some(meta)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &EntityMeta {
        self.meta.get_unchecked(index) 
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut EntityMeta {
        self.meta.get_unchecked_mut(index) 
    }

    #[inline]
    pub fn free_count(&self) -> usize {
        self.free_count
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.meta.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &EntityMeta> {
        self.meta.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut EntityMeta> {
        self.meta.iter_mut()
    }
}

