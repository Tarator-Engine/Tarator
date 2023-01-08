/// size of usize in bytes * bitsize of byte / 2
const BSIZE: usize = std::mem::size_of::<usize>() * 4;

pub(crate) trait IdTrait {
    fn new(index: Index, version: Version) -> Self;
    fn invalid() -> Self;
    fn versioned_invalid(version: Version) -> Self;
    fn get_index(self) -> Index;
    fn is_index_valid(self) -> bool;
    fn get_version(self) -> Version;
}

pub(crate) type Index = usize;
pub(crate) type Version = usize;
/// Layout is as follows when compiled on x64
/// [Index] (32 bits) [Version] (32 bits)
pub type Id = usize;

impl IdTrait for Id {
    #[inline]
    fn new(index: Index, version: Version) -> Self {
        (index << BSIZE) | version
    }
    #[inline]
    fn invalid() -> Self {
        Self::MAX 
    }
    #[inline]
    fn versioned_invalid(version: Version) -> Self {
        (Self::invalid() << BSIZE) | version
    }
    #[inline]
    fn get_index(self) -> Index {
        self >> BSIZE      
    }
    #[inline]
    fn is_index_valid(self) -> bool {
        self.get_index() != Index::MAX
    }
    #[inline]
    fn get_version(self) -> Version {
        (self << BSIZE) >> BSIZE
    }
}

