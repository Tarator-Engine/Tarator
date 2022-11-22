pub(crate) trait EntityIdTrait {
    fn new(index: EntityIndex, version: EntityVersion) -> Self;
    fn invalid() -> Self;
    fn versioned_invalid(version: EntityVersion) -> Self;
    fn index(self) -> EntityIndex;
    fn version(self) -> EntityVersion;
}

pub(crate) type EntityIndex = u32;
pub(crate) type EntityVersion = u32;
pub(crate) type EntityId = u64;

impl EntityIdTrait for EntityId {
    #[inline]
    fn new(index: EntityIndex, version: EntityVersion) -> Self {
        ((index as EntityId) << 32) | (version as EntityId)
    }
    #[inline]
    fn invalid() -> Self {
        Self::MAX
    }
    #[inline]
    fn versioned_invalid(version: EntityVersion) -> Self {
        Self::new(EntityIndex::MAX, version) 
    }
    #[inline]
    fn index(self) -> EntityIndex {
        (self >> 32) as EntityIndex
    }
    #[inline]
    fn version(self) -> EntityVersion {
        self as EntityVersion 
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Entity {
    id: EntityId
}

impl Entity {
    #[inline]
    pub(crate) fn new(id: EntityId) -> Self {
        Self { id }
    }
    #[inline]
    pub(crate) fn id(&self) -> EntityId {
        self.id
    }
}

