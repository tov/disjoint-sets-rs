use std::fmt::{self, Debug};
use std::marker::{Send, Sync};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free, concurrent union-find representing a set of disjoint sets.
///
/// # Warning
///
/// I don’t yet have good reason to believe that this is correct.
pub struct AUnionFind(Box<[Entry]>);

struct Entry {
    id:   AtomicUsize,
    rank: AtomicUsize,
}

unsafe impl Send for AUnionFind {}
unsafe impl Sync for AUnionFind {}

impl Clone for AUnionFind {
    fn clone(&self) -> Self {
        fn copy_slice(slice: &[Entry]) -> Box<[Entry]> {
            let mut vec = Vec::with_capacity(slice.len());
            for entry in slice {
                vec.push(Entry::new(entry.id.load(Ordering::SeqCst),
                                    entry.rank.load(Ordering::SeqCst)));
            }
            vec.into_boxed_slice()
        }

        AUnionFind(copy_slice(&*self.0))
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
    pub fn equiv(&self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    /// Forces all laziness, so that each element points directly to its
    /// set’s representative.
    pub fn force(&self) {
        for i in 0 .. self.len() {
            self.find(i);
        }
    }

    /// Returns a vector of set representatives.
    pub fn as_vec(&self) -> Vec<usize> {
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
}
