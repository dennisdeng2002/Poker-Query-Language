use std::{
    array,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    iter,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A fixed-capacity, variable-length vector of `Copy` values stored entirely on
/// the stack. Holds up to `N` elements; pushing beyond `N` panics rather than
/// spilling to the heap.
pub struct StackVec<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    /// Creates an empty vector.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// Returns the number of live elements.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if there are no live elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the elements as a slice.
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.buf.as_ptr().cast::<T>(), self.len) }
    }

    /// Returns the elements as a mutable slice.
    #[must_use]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buf.as_mut_ptr().cast::<T>(), self.len) }
    }
}

impl<T: Copy, const N: usize> StackVec<T, N> {
    /// Appends `value` to the end of the vector.
    ///
    /// # Panics
    /// Panics if the vector is already at capacity (`N`).
    pub const fn push(&mut self, value: T) {
        assert!(self.len < N, "StackVec: capacity exceeded");
        self.buf[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    /// Removes and returns the last element, or `None` if empty.
    pub const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(unsafe { self.buf[self.len].assume_init() })
    }

    /// Builds a vector from a slice.
    ///
    /// # Panics
    /// Panics if `items.len() > N`.
    #[must_use]
    pub fn from_slice(items: &[T]) -> Self {
        let mut v = Self::new();
        for &item in items {
            v.push(item);
        }
        v
    }

    /// Builds a vector of `n` copies of `value`.
    ///
    /// # Panics
    /// Panics if `n > N`.
    #[must_use]
    pub fn from_elem(value: T, n: usize) -> Self {
        let mut v = Self::new();
        for _ in 0..n {
            v.push(value);
        }
        v
    }

    #[must_use]
    #[inline]
    pub(crate) const fn from_arr<const M: usize>(arr: [T; M]) -> Self {
        const { assert!(M <= N, "from_arr: M must be <= N") };

        let mut v = Self::new();
        let mut i = 0;
        while i < M {
            v.buf[i] = MaybeUninit::new(arr[i]);
            i += 1;
        }
        v.len = M;
        v
    }
}

impl<T: Copy + Ord, const N: usize> StackVec<T, N> {
    /// Collects `items` into a vector sorted into ascending order.
    #[must_use]
    pub fn sorted(items: impl IntoIterator<Item = T>) -> Self {
        let mut v: Self = items.into_iter().collect();
        v.as_mut_slice().sort_unstable();
        v
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<T: Copy, const N: usize> Clone for StackVec<T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy, const N: usize> Copy for StackVec<T, N> {}

impl<T: fmt::Debug, const N: usize> fmt::Debug for StackVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: PartialEq, const N: usize> PartialEq for StackVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const N: usize> Eq for StackVec<T, N> {}

impl<T: Hash, const N: usize> Hash for StackVec<T, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T: PartialOrd, const N: usize> PartialOrd for StackVec<T, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Ord, const N: usize> Ord for StackVec<T, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T, const N: usize> Deref for StackVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for StackVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Index<usize> for StackVec<T, N> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.as_slice()[idx]
    }
}

impl<T, const N: usize> IndexMut<usize> for StackVec<T, N> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[idx]
    }
}

impl<T: Copy, const N: usize> From<Vec<T>> for StackVec<T, N> {
    fn from(items: Vec<T>) -> Self {
        Self::from_slice(&items)
    }
}

impl<T: Copy, const N: usize, const M: usize> From<[T; M]> for StackVec<T, N> {
    fn from(items: [T; M]) -> Self {
        Self::from_arr(items)
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for StackVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v = Self::new();
        for item in iter {
            v.push(item);
        }
        v
    }
}

const fn assume_init<T>(slot: MaybeUninit<T>) -> T {
    unsafe { slot.assume_init() }
}

impl<T: Copy, const N: usize> IntoIterator for StackVec<T, N> {
    type Item = T;
    type IntoIter =
        iter::Map<iter::Take<array::IntoIter<MaybeUninit<T>, N>>, fn(MaybeUninit<T>) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf
            .into_iter()
            .take(self.len)
            .map(assume_init as fn(MaybeUninit<T>) -> T)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a StackVec<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut StackVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize, const N: usize> Serialize for StackVec<T, N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + Deserialize<'de>, const N: usize> Deserialize<'de> for StackVec<T, N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let items = Vec::<T>::deserialize(deserializer)?;
        Ok(Self::from(items))
    }
}

#[cfg(feature = "speedy")]
impl<T, Ctx, const N: usize> speedy::Writable<Ctx> for StackVec<T, N>
where
    Ctx: speedy::Context,
    T: speedy::Writable<Ctx>,
{
    fn write_to<W: ?Sized + speedy::Writer<Ctx>>(&self, writer: &mut W) -> Result<(), Ctx::Error> {
        self.as_slice().write_to(writer)
    }

    fn bytes_needed(&self) -> Result<usize, Ctx::Error> {
        speedy::Writable::<Ctx>::bytes_needed(self.as_slice())
    }
}

#[cfg(feature = "speedy")]
impl<'a, T, Ctx, const N: usize> speedy::Readable<'a, Ctx> for StackVec<T, N>
where
    Ctx: speedy::Context,
    T: Copy + speedy::Readable<'a, Ctx>,
{
    fn read_from<R: speedy::Reader<'a, Ctx>>(reader: &mut R) -> Result<Self, Ctx::Error> {
        let items: Vec<T> = speedy::Readable::read_from(reader)?;
        Ok(Self::from(items))
    }

    fn minimum_bytes_needed() -> usize {
        4
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn push_pop_and_slice() {
        let mut v: StackVec<u8, 4> = StackVec::new();
        assert!(v.is_empty());
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.as_slice(), &[1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.as_slice(), &[1, 2]);
    }

    #[test]
    fn constructors() {
        assert_eq!(StackVec::<u8, 4>::from_elem(7, 3).as_slice(), &[7, 7, 7]);
        assert_eq!(StackVec::<u8, 4>::from_slice(&[1, 2]).as_slice(), &[1, 2]);
        assert_eq!(StackVec::<u8, 4>::from([9, 8]).as_slice(), &[9, 8]);
        assert_eq!(StackVec::<u8, 4>::from(vec![5, 6]).as_slice(), &[5, 6]);

        let collected: StackVec<u8, 4> = [3, 1, 2].into_iter().collect();
        assert_eq!(collected.as_slice(), &[3, 1, 2]);
        assert_eq!(StackVec::<u8, 4>::sorted([3, 1, 2]).as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn iteration() {
        let v: StackVec<u8, 4> = StackVec::from_slice(&[1, 2, 3]);
        assert_eq!((&v).into_iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(v.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn ordering_and_eq() {
        let a: StackVec<u8, 4> = StackVec::from_slice(&[1, 2]);
        let b: StackVec<u8, 4> = StackVec::from_slice(&[1, 2, 3]);
        assert!(a < b);
        assert_eq!(a, StackVec::<u8, 4>::from_slice(&[1, 2]));
    }

    #[test]
    #[should_panic(expected = "capacity exceeded")]
    fn push_beyond_capacity_panics() {
        let mut v: StackVec<u8, 2> = StackVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
    }

    #[test]
    fn default_and_pop_empty() {
        let mut v: StackVec<u8, 4> = StackVec::default();
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn clone_and_debug() {
        let v: StackVec<u8, 4> = StackVec::from_slice(&[1, 2]);
        assert_eq!(Clone::clone(&v), v);
        assert_eq!(format!("{v:?}"), "[1, 2]");
    }

    #[test]
    fn index_deref_and_iter_mut() {
        let mut v: StackVec<u8, 4> = StackVec::from_slice(&[1, 2]);
        assert_eq!(v[0], 1);
        v[0] = 9;
        assert!(v.contains(&2));
        v.reverse();
        for x in &mut v {
            *x += 1;
        }
        assert_eq!(v.as_slice(), &[3, 10]);
    }

    #[test]
    fn ord_and_hash() {
        use std::collections::hash_map::DefaultHasher;

        let a: StackVec<u8, 4> = StackVec::from_slice(&[1, 2]);
        let b: StackVec<u8, 4> = StackVec::from_slice(&[1, 2, 3]);
        assert_eq!(a.cmp(&b), Ordering::Less);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }
}
