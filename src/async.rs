use std::fmt::{self, Debug};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Concurrent union-find representing a set of disjoint sets.
///
/// # Warning
///
/// I don’t yet have good reason to believe that this is correct.
pub struct AUnionFind {
    elements: Box<[AtomicUsize]>,
    ranks:    Box<[AtomicUsize]>,
}

impl Debug for AUnionFind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "AUnionFind({:?})", self.elements)
    }
}

impl AUnionFind {
    /// Creates a new asynchronous union-find of `size` elements.
    pub fn new(size: usize) -> Self {
        let elements = (0..size).map(AtomicUsize::new).collect::<Vec<_>>();
        let ranks = (0..size).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();
        AUnionFind {
            elements: elements.into_boxed_slice(),
            ranks:    ranks.into_boxed_slice(),
        }
    }

    /// The number of elements in all the sets.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Is the union-find devoid of elements?
    ///
    /// It is possible to create an empty `AUnionFind`, but unlike with
    /// [`UnionFind`](struct.UnionFind.html) it is not possible to add
    /// elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Joins the sets of the two given elements.
    pub fn union(&self, mut a: usize, mut b: usize) {
        loop {
            a = self.find(a);
            b = self.find(b);

            if a == b { return }

            let rank_a = self.rank(a);
            let rank_b = self.rank(b);

            if rank_a > rank_b {
                if self.change_parent(b, b, a) { return }
            } else if rank_b > rank_a {
                if self.change_parent(a, a, b) { return }
            } else if self.change_parent(a, a, b) {
                self.increment_rank(b);
                return;
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
        self.elements.iter().map(|v| v.load(Ordering::SeqCst)).collect()
    }

    // HELPERS

    fn rank(&self, element: usize) -> usize {
        self.ranks[element].load(Ordering::SeqCst)
    }

    fn increment_rank(&self, element: usize) {
        self.ranks[element].fetch_add(1, Ordering::SeqCst);
    }

    fn parent(&self, element: usize) -> usize {
        self.elements[element].load(Ordering::SeqCst)
    }

    fn change_parent(&self,
                     element: usize,
                     old_parent: usize,
                     new_parent: usize)
                     -> bool {
        self.elements[element].compare_and_swap(old_parent,
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
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(4, 3);
        uf.union(3, 2);
        assert!(uf.equiv(0, 1));
        assert!(uf.equiv(0, 2));
        assert!(uf.equiv(0, 3));
        assert!(uf.equiv(0, 4));
        assert!(!uf.equiv(0, 5));

        uf.union(5, 3);
        assert!(uf.equiv(0, 5));

        uf.union(6, 7);
        assert!(uf.equiv(6, 7));
        assert!(!uf.equiv(5, 7));

        uf.union(0, 7);
        assert!(uf.equiv(5, 7));
    }
}
