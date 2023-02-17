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


/// Saves the component location of an [`Entity`], as well as it's current version. Every time an
/// [`Entity`] gets deleted, the corresponding [`EntityMeta`] gets invalidated and the version gets
/// incremented (in order to recycle it's existanse).
#[derive(Debug)]
pub struct EntityMeta {
    pub archetype_id: ArchetypeId,
    pub index: usize,
    pub version: u32
}

impl EntityMeta {
    #[inline]
    pub const fn new() -> Self {
        Self {
            archetype_id: ArchetypeId::INVALID,
            version: 0,
            index: 0
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetype_id == ArchetypeId::EMPTY
    }
}


/// Stores all the data information of our entities and checks if they are still alive. The result
/// of [`Entity::id()`] directly maps to a corresponding [`EntityMeta`], but
/// [`EntityMeta.version`] has to match [`Entity::version()`] and [`EntityMeta.archetype_id`] has
/// to be valid.
///
/// Every corresponding [`EntityMeta`] gets set in the [`World`](crate::world::World) via a mutable
/// reference.
///
/// # Links
///
/// [ECS back and forth - Part 3](https://skypjack.github.io/2019-05-06-ecs-baf-part-3)
#[derive(Debug)]
pub struct Entities {
    meta: Vec<EntityMeta>,
    /// `free_next` is pointing to the next dead [`Entity`] that can get revived, and `free_count`
    /// stores how many [`Entity`]s are currently dead. On dead [`Entity`]s, the corresponding
    /// [`EntityMeta.index`] points to the next dead [`Entity`], making a linked list of sorts.
    /// This way, [`Entity`]s can easily get reused after they get destroyed, without the need of
    /// reallocating the array for every [`Entity`] that gets created.
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

    /// Instantiates a new [`EntityMeta`] or revives one, and returns a mutable reference which can
    /// be used to set the location to the [`Entity`]'s [`Component`](crate::component::Component)
    /// in the [`World`](crate::world::World).
    pub fn create(&mut self) -> (Entity, &mut EntityMeta) {
        if self.free_count == 0 {
            let index = self.meta.len();
            self.meta.push(EntityMeta::new());

            // SAFETY:
            // Meta was just pushed in
            return (Entity::new(index as u32, 0), unsafe { self.meta.get_unchecked_mut(index) });
        }

        let index = self.free_next;

        let free = &mut self.meta[self.free_next];
        // Free entities should have an [`ArchetypeId`] variant [`ArchetypeId::INVALID`]
        debug_assert!(free.archetype_id == ArchetypeId::INVALID);

        // set `free_next` to the index our free had pointed to
        self.free_next = free.index;
        self.free_count -= 1;

        // Set our freed [`EntityMeta`]
        free.archetype_id = ArchetypeId::EMPTY;
        free.index = 0;

        // SAFETY:
        // Index was saved by free_next
        (Entity::new(index as u32, free.version), unsafe { self.meta.get_unchecked_mut(index) })
    }

    /// Returns the destroyed [`EntityMeta`] which can be used to drop all
    /// [`Component`](crate::component::Component)s of given [`Entity`]. Will return [`None`] if
    /// the [`Entity`] was already destroyed or revived.
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

        // Set the index of our [`EntityMeta`] that we want to destory to the current `free_next`,
        // and set `free_next` to our index, as well as increment `free_count`. Also increment the
        // current version of our [`EntityMeta`].
        meta.index = self.free_next;
        meta.version += 1;
        self.free_next = index;
        self.free_count += 1;

        meta.archetype_id = ArchetypeId::INVALID;

        Some(old_meta)
    }

    /// Returns [`None`] if the [`Entity`] was already destroyed or revived.
    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&EntityMeta> {
        let meta = &self.meta[entity.id() as usize];

        if meta.version != entity.version() || meta.archetype_id == ArchetypeId::INVALID {
            return None;
        }

        Some(meta)
    }

    /// Returns [`None`] if the [`Entity`] was already destroyed or revived.
    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityMeta> {
        let meta = self.meta.get_mut(entity.id() as usize)?;

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

    /// Returns how many [`Entity`]s are currently dead
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

