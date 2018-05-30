use std::fmt::{self, Debug};
use std::marker::{Send, Sync};
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};

/// Lock-free, concurrent union-find representing a set of disjoint sets.
///
/// # Warning
///
/// I don’t yet have good reason to believe that this is correct.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AUnionFind(Box<[Entry]>);

struct Entry {
    id:   AtomicUsize,
    rank: AtomicUsize,
}

unsafe impl Send for AUnionFind {}
unsafe impl Sync for AUnionFind {}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Entry::new(self.id.load(Ordering::SeqCst),
                   self.rank.load(Ordering::SeqCst))
    }
}

impl Debug for AUnionFind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "AUnionFind(")?;
        formatter.debug_list()
            .entries(self.0.iter().map(|entry| &entry.id)).finish()?;
        write!(formatter, ")")
    }
}

impl Default for AUnionFind {
    fn default() -> Self {
        AUnionFind::new(0)
    }
}

impl Entry {
    fn new(id: usize, rank: usize) -> Self {
        Entry {
            id:   AtomicUsize::new(id),
            rank: AtomicUsize::new(rank),
        }
    }
}

impl AUnionFind {
    /// Creates a new asynchronous union-find of `size` elements.
    pub fn new(size: usize) -> Self {
        AUnionFind((0..size)
            .map(|i| Entry::new(i, 0))
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
    pub fn union(&self, mut a: usize, mut b: usize) -> bool {
        loop {
            a = self.find(a);
            b = self.find(b);

            if a == b { return false; }

            let rank_a = self.rank(a);
            let rank_b = self.rank(b);

            if rank_a > rank_b {
                if self.change_parent(b, b, a) { return true; }
            } else if rank_b > rank_a {
                if self.change_parent(a, a, b) { return true; }
            } else if self.change_parent(a, a, b) {
                self.increment_rank(b);
                return true;
            }
        }
    }

    /// Finds the representative element for the given element’s set.
    pub fn find(&self, mut element: usize) -> usize {
        let mut parent = self.parent(element);

        while element != parent {
            let grandparent = self.parent(parent);
            self.change_parent(element, parent, grandparent);
            element = parent;
            parent = grandparent;
        }

        element
    }

    /// Determines whether two elements are in the same set.
    pub fn equiv(&self, mut a: usize, mut b: usize) -> bool {
        loop {
            a = self.find(a);
            b = self.find(b);

            if a == b { return true; }
            if self.parent(a) == a { return false; }
        }
    }

    /// Forces all laziness, so that each element points directly to its
    /// set’s representative.
    pub fn force(&self) {
        for i in 0 .. self.len() {
            loop {
                let parent = self.parent(i);
                if i == parent {
                    break
                } else {
                    let root = self.find(parent);
                    if parent == root || self.change_parent(i, parent, root) {
                        break;
                    }
                }
            }
        }
    }

    /// Returns a vector of set representatives.
    pub fn to_vec(&self) -> Vec<usize> {
        self.force();
        self.0.iter().map(|entry| entry.id.load(Ordering::SeqCst)).collect()
    }

    // HELPERS

    fn rank(&self, element: usize) -> usize {
        self.0[element].rank.load(Ordering::SeqCst)
    }

    fn increment_rank(&self, element: usize) {
        self.0[element].rank.fetch_add(1, Ordering::SeqCst);
    }

    fn parent(&self, element: usize) -> usize {
        self.0[element].id.load(Ordering::SeqCst)
    }

    fn change_parent(&self,
                     element: usize,
                     old_parent: usize,
                     new_parent: usize)
                     -> bool {
        self.0[element].id.compare_and_swap(old_parent,
                                            new_parent,
                                            Ordering::SeqCst)
            == old_parent
    }
}

#[cfg(feature = "serde")]
impl Serialize for Entry {
    fn serialize<S: Serializer>(&self, serializer: S)
                                -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    {
        use serde::ser::SerializeStruct;

        let mut tuple = serializer.serialize_struct("Entry", 2)?;
        tuple.serialize_field("id", &self.id.load(Ordering::Relaxed))?;
        tuple.serialize_field("rank", &self.rank.load(Ordering::Relaxed))?;
        tuple.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, Visitor, SeqAccess, MapAccess};

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Id, Rank, }

        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = Entry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Entry")
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Self::Value, V::Error> {
                let id = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let rank = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Entry::new(id, rank))
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut id   = None;
                let mut rank = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Rank => {
                            if rank.is_some() {
                                return Err(de::Error::duplicate_field("rank"));
                            }
                            rank = Some(map.next_value()?);
                        }
                    }
                }

                let id   = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let rank = rank.ok_or_else(|| de::Error::missing_field("rank"))?;

                Ok(Entry::new(id, rank))
            }
        }

        const FIELDS: &'static [&'static str] = &["id", "rank"];
        deserializer.deserialize_struct("Entry", FIELDS, EntryVisitor)
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
