use std::cell::Cell;

use super::ElementType;

/// Array-based union-find representing a set of disjoint sets.
#[derive(Clone, Debug)]
pub struct UnionFind<E: ElementType = usize> {
    elements: Vec<Cell<E>>,
    ranks: Vec<u8>,
}

impl<E: ElementType> UnionFind<E> {
    /// Creates a new union-find of `size` elements.
    ///
    /// # Panics
    ///
    /// If `size` elements would overflow the element type `E`.
    pub fn new(size: usize) -> Self {
        UnionFind {
            elements: (0..size).map(|i| {
                let e = E::from_usize(i).expect("UnionFind::new: overflow");
                Cell::new(e)
            }).collect(),
            ranks: vec![0; size],
        }
    }

    /// The number of elements in all the sets.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Creates a new element in a singleton set.
    ///
    /// # Panics
    ///
    /// If allocating another element would overflow the element type
    /// `E`.
    pub fn alloc(&mut self) -> E {
        let result = E::from_usize(self.elements.len())
                       .expect("UnionFind::alloc: overflow");
        self.elements.push(Cell::new(result));
        self.ranks.push(0);
        result
    }

    /// Joins the sets of the two given elements.
    pub fn union(&mut self, a: E, b: E) {
        let a = self.find(a);
        let b = self.find(b);

        if a == b { return }

        let rank_a = self.rank(a);
        let rank_b = self.rank(b);

        if rank_a > rank_b {
            self.set_parent(b, a);
        } else if rank_b > rank_a {
            self.set_parent(a, b);
        } else {
            self.set_parent(a, b);
            self.increment_rank(b);
        }
    }

    /// Finds the representative element for the given element’s set.
    pub fn find(&self, mut element: E) -> E {
        while element != self.parent(element) {
            self.set_parent(element, self.grandparent(element));
            element = self.parent(element);
        }

        element
    }

    /// Determines whether two elements are in the same set.
    pub fn equiv(&self, a: E, b: E) -> bool {
        self.find(a) == self.find(b)
    }

    /// Forces all laziness, so that each element points directly to its
    /// set’s representative.
    pub fn force(&self) {
        for i in 0 .. self.len() {
            self.find(E::from_usize(i).unwrap());
        }
    }

    /// Returns a vector of set representatives.
    pub fn as_vec(&self) -> Vec<E> {
        self.force();
        self.elements.iter().map(Cell::get).collect()
    }

    // HELPERS

    fn rank(&self, element: E) -> u8 {
        self.ranks[element.to_usize()]
    }

    fn increment_rank(&mut self, element: E) {
        let i = element.to_usize();
        let (rank, over) = self.ranks[i].overflowing_add(1);
        assert!(!over, "UnionFind: rank overflow");
        self.ranks[i] = rank;
    }

    fn parent(&self, element: E) -> E {
        self.elements[element.to_usize()].get()
    }

    fn set_parent(&self, element: E, parent: E) {
        self.elements[element.to_usize()].set(parent);
    }

    fn grandparent(&self, element: E) -> E {
        self.parent(self.parent(element))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len() {
        assert_eq!(5, UnionFind::<u32>::new(5).len());
    }

    #[test]
    fn union() {
        let mut uf = UnionFind::<u32>::new(8);
        assert!(!uf.equiv(0, 1));
        uf.union(0, 1);
        assert!(uf.equiv(0, 1));
    }

    #[test]
    fn unions() {
        let mut uf = UnionFind::<usize>::new(8);
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
