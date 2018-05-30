use std::cell::Cell;
use std::fmt::{self, Debug};

use super::ElementType;

/// Vector-based union-find representing a set of disjoint sets.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnionFind<Element: ElementType = usize> {
    elements: Vec<Cell<Element>>,
    ranks: Vec<u8>,
}
// Invariant: self.elements.len() == self.ranks.len()

impl<Element: Debug + ElementType> Debug for UnionFind<Element> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "UnionFind({:?})", self.elements)
    }
}

impl<Element: ElementType> Default for UnionFind<Element> {
    fn default() -> Self {
        UnionFind::new(0)
    }
}

impl<Element: ElementType> UnionFind<Element> {
    /// Creates a new union-find of `size` elements.
    ///
    /// # Panics
    ///
    /// If `size` elements would overflow the element type `Element`.
    pub fn new(size: usize) -> Self {
        UnionFind {
            elements: (0..size).map(|i| {
                let e = Element::from_usize(i).expect("UnionFind::new: overflow");
                Cell::new(e)
            }).collect(),
            ranks: vec![0; size],
        }
    }

    /// The number of elements in all the sets.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Is the union-find devoid of elements?
    ///
    /// It is possible to create an empty `UnionFind` and then add
    /// elements with [`alloc`](#method.alloc).
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Creates a new element in a singleton set.
    ///
    /// # Panics
    ///
    /// If allocating another element would overflow the element type
    /// `Element`.
    pub fn alloc(&mut self) -> Element {
        let result = Element::from_usize(self.elements.len())
                       .expect("UnionFind::alloc: overflow");
        self.elements.push(Cell::new(result));
        self.ranks.push(0);
        result
    }

    /// Joins the sets of the two given elements.
    ///
    /// Returns whether anything changed. That is, if the sets were
    /// different, it returns `true`, but if they were already the same
    /// then it returns `false`.
    pub fn union(&mut self, a: Element, b: Element) -> bool {
        let a = self.find(a);
        let b = self.find(b);

        if a == b { return false; }

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

        true
    }

    /// Finds the representative element for the given element’s set.
    pub fn find(&self, mut element: Element) -> Element {
        let mut parent = self.parent(element);

        while element != parent {
            let grandparent = self.parent(parent);
            self.set_parent(element, grandparent);
            element = parent;
            parent = grandparent;
        }

        element
    }

    /// Determines whether two elements are in the same set.
    pub fn equiv(&self, a: Element, b: Element) -> bool {
        self.find(a) == self.find(b)
    }

    /// Forces all laziness, so that each element points directly to its
    /// set’s representative.
    pub fn force(&self) {
        for i in 0 .. self.len() {
            let element = Element::from_usize(i).unwrap();
            let root = self.find(element);
            self.set_parent(element, root);
        }
    }

    /// Returns a vector of set representatives.
    pub fn to_vec(&self) -> Vec<Element> {
        self.force();
        self.elements.iter().map(Cell::get).collect()
    }

    // HELPERS

    fn rank(&self, element: Element) -> u8 {
        self.ranks[element.to_usize()]
    }

    fn increment_rank(&mut self, element: Element) {
        let i = element.to_usize();
        self.ranks[i] = self.ranks[i].saturating_add(1);
    }

    fn parent(&self, element: Element) -> Element {
        self.elements[element.to_usize()].get()
    }

    fn set_parent(&self, element: Element, parent: Element) {
        self.elements[element.to_usize()].set(parent);
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

        uf.union(5, 3);
        assert!(uf.equiv(0, 5));

        uf.union(6, 7);
        assert!(uf.equiv(6, 7));
        assert!(!uf.equiv(5, 7));

        uf.union(0, 7);
        assert!(uf.equiv(5, 7));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        extern crate serde_json;

        let mut uf0: UnionFind<usize> = UnionFind::new(8);
        uf0.union(0, 1);
        uf0.union(2, 3);
        assert!( uf0.equiv(0, 1));
        assert!(!uf0.equiv(1, 2));
        assert!( uf0.equiv(2, 3));

        let json = serde_json::to_string(&uf0).unwrap();
        let uf1: UnionFind<usize> = serde_json::from_str(&json).unwrap();
        assert!( uf1.equiv(0, 1));
        assert!(!uf1.equiv(1, 2));
        assert!( uf1.equiv(2, 3));
    }
}
