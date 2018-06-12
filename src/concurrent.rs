use std::fmt::{self, Debug};
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};

// How many bits should we use for id, and how many for rank? In the worst
// case, we can build a tree of rank 0 using 1 node, and a tree of rank
// *N* + 1 using two trees of rank *N*. So:
//
//   - Nodes(0) = 1
//   - Nodes(*N* + 1) = 2 * Nodes(*N*)
//
//  In closed form, Nodes(*N*) = 2^*N*. So suppose we reserve *R* bits for
//  rank. Then the maximum rank is 2^*R* - 1. So to reach that rank, we need
//  2^(2^*R* - 1) nodes, which we can reach at 2^*R* - 1 bits per index.
//
//  With 64 bits to allot, suppose that *R* is 8. Then we can accommodate ranks
//  up to 255, but since we are limited to 2^56 objects, the highest rank we can
//  attain is 56.. Let *R* be 6. Then ranks have room for 63. But with only
//  58 remaining bits to play with, ranks cannot exceed 58.
//
//  For a 32-bit platform, let *R* be 5. Then we can accomodate ranks up to 32,
//  which is safe for the remaining 27 bits that we will use for ids.

#[cfg(target_pointer_width = "64")]
const ID_BITS: usize = 58;
#[cfg(target_pointer_width = "64")]
const RK_BITS: usize = 64 - ID_BITS;

#[cfg(target_pointer_width = "32")]
const ID_BITS: usize = 27;
#[cfg(target_pointer_width = "32")]
const RK_BITS: usize = 32 - ID_BITS;

const RK_SHIFT: usize = 0;
const ID_SHIFT: usize = RK_BITS;

const RK_MASK: usize = (1 << RK_BITS) - 1;
const ID_MASK: usize = !RK_MASK;

const RK_MAX: usize = 1 << RK_BITS;
const ID_MAX: usize = 1 << ID_BITS;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Entry {
    id: usize,
    rk: usize,
}

impl From<usize> for Entry {
    fn from(value: usize) -> Self {
        let id = (value & ID_MASK) >> ID_SHIFT;
        let rk = (value & RK_MASK) >> RK_SHIFT;
        Entry { id: id, rk: rk, }
    }
}

impl From<Entry> for usize {
    fn from(view: Entry) -> Self {
        (view.id << ID_SHIFT) | (view.rk << RK_SHIFT)
    }
}

impl Entry {
    fn new(id: usize) -> Self {
        Self::with_rank(id, 0)
    }

    fn with_rank(id: usize, rk: usize) -> Self {
        debug_assert!( id < ID_MAX );
        debug_assert!( rk < RK_MAX );
        Entry {
            id: id,
            rk: rk,
        }
    }

    fn inc_rank(self) -> Self {
        Entry::with_rank(self.id, self.rk + 1)
    }
}

struct AtomicEntry(AtomicUsize);

impl Clone for AtomicEntry {
    fn clone(&self) -> Self {
        AtomicEntry(AtomicUsize::new(self.0.load(Ordering::Relaxed)))
    }
}

impl From<Entry> for AtomicEntry {
    fn from(view: Entry) -> Self {
        let value = usize::from(view);
        AtomicEntry(AtomicUsize::new(value))
    }
}

impl AtomicEntry {
    fn new(id: usize) -> Self {
        Self::from(Entry::new(id))
    }

    fn load(&self, ordering: Ordering) -> Entry {
        let value = self.0.load(ordering);
        Entry::from(value)
    }

    fn compare_and_swap(&self, exp: Entry, new: Entry,
                        ordering: Ordering) -> bool {

        let exp_value = usize::from(exp);
        let new_value = usize::from(new);
        let old_value = self.0.compare_and_swap(exp_value, new_value, ordering);
        exp_value == old_value
    }
}

/// Lock-free, concurrent union-find representing a set of disjoint sets.
///
/// If configured with Cargo feature `"serde"`, impls for `Serialize`
/// and `Deserialize` will be defined. Note that if the union-find is
/// modified while being serialized, the view of the structure
/// preserved by may not correspond to any particular moment in time.
///
/// # Warning
///
/// This should always produce correct answers, but the expected complexity
/// guarantees may not hold.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AUnionFind(Box<[AtomicEntry]>);

impl Debug for AUnionFind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "AUnionFind(")?;
        formatter.debug_list()
            .entries(self.0.iter().map(|entry| entry.load(Ordering::Relaxed).id)).finish()?;
        write!(formatter, ")")
    }
}

impl Default for AUnionFind {
    fn default() -> Self {
        AUnionFind::new(0)
    }
}

impl AUnionFind {
    /// The maximum number of elements of an `AUnionFind`.
    pub fn max_size() -> usize {
        ID_MAX
    }

    /// Creates a new asynchronous union-find of `size` elements.
    ///
    /// # Panics
    ///
    /// If `size > Self::max_size()`.
    pub fn new(size: usize) -> Self {
        assert!(size <= Self::max_size());
        AUnionFind((0..size)
            .map(AtomicEntry::new)
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    /// The number of elements in all the sets.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is the union-find devoid of elements?
    ///
    /// It is possible to create an empty `AUnionFind`, but unlike with
    /// [`UnionFind`](struct.UnionFind.html) it is not possible to add
    /// elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Joins the sets of the two given elements.
    ///
    /// Returns whether anything changed. That is, if the sets were
    /// different, it returns `true`, but if they were already the same
    /// then it returns `false`.
    pub fn union(&self, mut a_id: usize, mut b_id: usize) -> bool {
        loop {
            let a = self.find_entry(a_id);
            let b = self.find_entry(b_id);

            if a.id == b.id { return false; }

            if a.rk > b.rk {
                if self.compare_and_swap(b.id, b, a) { return true; }
            } else if b.rk > a.rk {
                if self.compare_and_swap(a.id, a, b) { return true; }
            } else if self.compare_and_swap(a.id, a, b) {
                self.increment_rank(b);
                return true;
            }

            a_id = a.id;
            b_id = b.id;
        }
    }

    /// Finds the representative element for the given element’s set.
    pub fn find(&self, element: usize) -> usize {
        self.find_entry(element).id
    }

    /// Determines whether two elements are in the same set.
    pub fn equiv(&self, mut a: usize, mut b: usize) -> bool {
        loop {
            a = self.find(a);
            b = self.find(b);

            if a == b { return true; }
            if self.load(a).id == a { return false; }
        }
    }

    /// Forces all laziness, so that each element points directly to its
    /// set’s representative.
    pub fn force(&self) {
        for i in 0 .. self.len() {
            loop {
                let parent = self.load(i);
                if i == parent.id {
                    break
                } else {
                    let root = self.find_entry(parent.id);
                    if parent.id == root.id || self.compare_and_swap(i, parent, root) {
                        break;
                    }
                }
            }
        }
    }

    /// Returns a vector of set representatives.
    pub fn to_vec(&self) -> Vec<usize> {
        self.force();
        self.0.iter().map(|entry| entry.load(Ordering::SeqCst).id).collect()
    }

    // HELPERS

    // Note that increment_rank can fail to CAS, but this should be okay,
    // because the only ways it can fail are if 1) the id of the entry
    // changed, in which case its rank doesn't matter any more, or 2)
    // the rank changed, in which case it has already been incremented.
    fn increment_rank(&self, entry: Entry) {
        self.0[entry.id].compare_and_swap(entry,
                                          entry.inc_rank(),
                                          Ordering::SeqCst);
    }

    fn load(&self, element: usize) -> Entry {
        self.0[element].load(Ordering::SeqCst)
    }

    fn find_entry(&self, mut element: usize) -> Entry {
        let mut parent = self.load(element);

        while element != parent.id {
            let grandparent = self.load(parent.id);
            self.compare_and_swap(element, parent, grandparent);
            element = parent.id;
            parent = grandparent;
        }

        parent
    }

    fn compare_and_swap(&self,
                        index: usize,
                        exp_entry: Entry,
                        new_entry: Entry)
                        -> bool {

        self.0[index].compare_and_swap(exp_entry, new_entry, Ordering::SeqCst)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AtomicEntry {
    fn serialize<S: Serializer>(&self, serializer: S)
                                -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    {
        let entry = self.load(Ordering::Relaxed);
        Entry::serialize(&entry, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AtomicEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Entry::deserialize(deserializer).map(AtomicEntry::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len() {
        assert_eq!(5, AUnionFind::new(5).len());
    }

    #[test]
    fn union() {
        let uf = AUnionFind::new(8);
        assert!(!uf.equiv(0, 1));
        uf.union(0, 1);
        assert!(uf.equiv(0, 1));
    }

    #[test]
    fn unions() {
        let uf = AUnionFind::new(8);
        assert!(uf.union(0, 1));
        assert!(uf.union(1, 2));

        assert!(uf.union(4, 3));
        assert!(uf.union(3, 2));
        assert!(! uf.union(0, 3));

        assert!(uf.equiv(0, 1));
        assert!(uf.equiv(0, 2));
        assert!(uf.equiv(0, 3));
        assert!(uf.equiv(0, 4));
        assert!(!uf.equiv(0, 5));

        assert!(uf.union(5, 3));
        assert!(uf.equiv(0, 5));

        assert!(uf.union(6, 7));
        assert!(uf.equiv(6, 7));
        assert!(!uf.equiv(5, 7));

        assert!(uf.union(0, 7));
        assert!(uf.equiv(5, 7));
    }

    #[test]
    fn changed() {
        let uf = AUnionFind::new(8);
        assert!(uf.union(2, 3));
        assert!(uf.union(0, 1));
        assert!(uf.union(1, 3));
        assert!(!uf.union(0, 2))
    }

    // This assumes that for equal-ranked roots, the first argument
    // to union is pointed to the second.
    #[test]
    fn to_vec() {
        let uf = AUnionFind::new(6);
        assert_eq!(uf.to_vec(), vec![0, 1, 2, 3, 4, 5]);
        uf.union(0, 1);
        assert_eq!(uf.to_vec(), vec![1, 1, 2, 3, 4, 5]);
        uf.union(2, 3);
        assert_eq!(uf.to_vec(), vec![1, 1, 3, 3, 4, 5]);
        uf.union(1, 3);
        assert_eq!(uf.to_vec(), vec![3, 3, 3, 3, 4, 5]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        extern crate serde_json;

        let uf0 = AUnionFind::new(8);
        uf0.union(0, 1);
        uf0.union(2, 3);
        assert!( uf0.equiv(0, 1));
        assert!(!uf0.equiv(1, 2));
        assert!( uf0.equiv(2, 3));

        let json = serde_json::to_string(&uf0).unwrap();
        let uf1: AUnionFind = serde_json::from_str(&json).unwrap();
        assert!( uf1.equiv(0, 1));
        assert!(!uf1.equiv(1, 2));
        assert!( uf1.equiv(2, 3));
    }
}
