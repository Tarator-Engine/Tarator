//! Big portions of this code where looked up from
//! <https://docs.rs/bevy_ecs/latest/src/bevy_ecs/storage/sparse_set.rs.html>

use std::{
    marker::PhantomData,
    hash::Hash
};


/// Something that can be stored in a [`SparceSet`]
pub trait SparseSetIndex: Clone + Eq + PartialEq + Hash {
    fn as_usize(&self) -> usize;
    fn from_usize(value: usize) -> Self;
}

macro_rules! impl_sparse_set_index {
    ($($ty:ty),+) => {
        $(impl SparseSetIndex for $ty {
            fn as_usize(&self) -> usize {
                *self as usize
            }
            fn from_usize(value: usize) -> Self {
                value as $ty
            }
        })*
    };
}

impl_sparse_set_index!(u8, u16, u32, u64, usize);


/// Immutable: values cannot be changed after construction
#[derive(Debug)]
pub struct SparseArray<I, V = I> {
    values: Box<[Option<V>]>,
    marker: PhantomData<I>
}

/// Mutable: values can be changed after construction
#[derive(Debug)]
pub struct MutSparseArray<I, V = I> {
    values: Vec<Option<V>>,
    marker: PhantomData<I>
}

macro_rules! impl_sparse_array {
    ($ty:ident) => {
        impl<I: SparseSetIndex, V> $ty<I, V> {
            #[inline]
            pub fn contains(&self, index: I) -> bool {
                let index = index.as_usize();
                self.values.get(index).map(|v| v.is_some()).unwrap_or(false)
            }

            #[inline]
            pub fn get(&self, index: I) -> Option<&V> {
                let index = index.as_usize();
                self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
            }
        }
    };
}

impl_sparse_array!(SparseArray);
impl_sparse_array!(MutSparseArray);

impl<I: SparseSetIndex, V> MutSparseArray<I, V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            marker: PhantomData
        }
    }

    #[inline]
    pub fn insert(&mut self, index: I, value: V) {
        let index = index.as_usize();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        let index = index.as_usize();
        self.values
            .get_mut(index)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn remove(&mut self, index: I) -> Option<V> {
        let index = index.as_usize();
        self.values.get_mut(index).and_then(|value| value.take())
    }

    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Converts this [`MutSparseArray`] into [`SparseArray`], making it immutable
    #[inline]
    pub fn lock(self) -> SparseArray<I, V> {
        SparseArray {
            values: self.values.into_boxed_slice(),
            marker: PhantomData
        }
    }
}


#[derive(Debug)]
pub struct SparseSet<I, V: 'static> {
    dense: Box<[V]>,
    indices: Box<[I]>,
    sparse: SparseArray<I, usize>
}


#[derive(Debug)]
pub struct MutSparseSet<I, V: 'static> {
    dense: Vec<V>,
    indices: Vec<I>,
    sparse: MutSparseArray<I, usize>
}


macro_rules! impl_sparse_set {
    ($ty:ident) => {
        impl<I: SparseSetIndex, V> $ty<I, V> {
            #[inline]
            pub fn len(&self) -> usize {
                self.dense.len()
            }

            #[inline]
            pub fn contains(&self, index: I) -> bool {
                self.sparse.contains(index)
            }

            pub fn get(&self, index: I) -> Option<&V> {
                self.sparse.get(index).map(|dense_index| unsafe { self.dense.get_unchecked(*dense_index) })
            }

            pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
                let dense = &mut self.dense;
                self.sparse.get(index).map(move |dense_index| unsafe { dense.get_unchecked_mut(*dense_index) })
            }

            pub fn indices(&self) -> impl Iterator<Item = I> + '_ {
                self.indices.iter().cloned()
            }

            pub fn values(&self) -> impl Iterator<Item = &V> {
                self.dense.iter()
            }

            pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
                self.dense.iter_mut()
            }

            pub fn iter(&self) -> impl Iterator<Item = (&I, &V)> {
                self.indices.iter().zip(self.dense.iter())
            }

            pub fn iter_mut(&mut self) -> impl Iterator<Item = (&I, &mut V)> {
                self.indices.iter().zip(self.dense.iter_mut())
            }
        } 
    };
}

impl_sparse_set!(SparseSet);
impl_sparse_set!(MutSparseSet);


impl<I: SparseSetIndex, V> MutSparseSet<I, V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            dense: Vec::new(),
            indices: Vec::new(),
            sparse: MutSparseArray::new()
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dense: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            sparse: MutSparseArray::new(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.dense.capacity()
    }

    pub fn insert(&mut self, index: I, value: V) {
        if let Some(dense_index) = self.sparse.get(index.clone()).cloned() {
            unsafe { *self.dense.get_unchecked_mut(dense_index) = value; }
        } else {
            self.sparse.insert(index.clone(), self.dense.len());
            self.indices.push(index);
            self.dense.push(value);
        }
    }

    pub fn get_or_insert_with(&mut self, index: I, func: impl FnOnce() -> V) -> &mut V {
        if let Some(dense_index) = self.sparse.get(index.clone()).cloned() {
            unsafe { self.dense.get_unchecked_mut(dense_index) }
        } else {
            let value = func();
            let dense_index = self.dense.len();
            self.sparse.insert(index.clone(), dense_index);
            self.indices.push(index);
            self.dense.push(value);
            unsafe { self.dense.get_unchecked_mut(dense_index) }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.dense.len() == 0
    }

    pub fn remove(&mut self, index: I) -> Option<V> {
        self.sparse.remove(index).map(|dense_index| {
            let is_last = dense_index == self.dense.len() - 1;
            let value = self.dense.swap_remove(dense_index);
            self.indices.swap_remove(dense_index);
            if !is_last {
                let swapped_index = self.indices[dense_index].clone();
                *self.sparse.get_mut(swapped_index).unwrap() = dense_index;
            }
            value
        })
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        self.indices.clear();
        self.sparse.clear();
    }

    /// Converts this [`MutSparseSet`] into [`SparseSet`], making it immutable
    pub fn lock(self) -> SparseSet<I, V> {
        SparseSet {
            dense: self.dense.into_boxed_slice(),
            indices: self.indices.into_boxed_slice(),
            sparse: self.sparse.lock(),
        }
    }
}

