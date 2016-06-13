//! Tree-based union-find with associated data.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::mem;

/// Tree-based union-find representing disjoint sets with associated data.
///
/// This union-find implementation uses nodes to represent set elements
/// in a parent-pointer tree. Each set has associated with it an object
/// of type `Data`, which can be looked up and modified via any
/// representative of the set.
///
/// Construct a new singleton set with [`UnionFindNode::new`](#method.new).
pub struct UnionFindNode<Data = ()>(Rc<RefCell<NodeImpl<Data>>>);

enum NodeImpl<Data> {
    Root {
        data: Data,
        rank: u8,
    },
    Link(UnionFindNode<Data>),
    Dummy,
}

use self::NodeImpl::*;

impl<Data> UnionFindNode<Data> {
    fn id(&self) -> usize {
        &*self.0 as *const _ as usize
    }
}

impl<Data> Debug for UnionFindNode<Data> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "UnionFindNode({:p})", self.0)
    }
}

impl<Data> PartialEq for UnionFindNode<Data> {
    fn eq(&self, other: &UnionFindNode<Data>) -> bool {
        self.id() == other.id()
    }
}

impl<Data> Eq for UnionFindNode<Data> { }

impl<Data> PartialOrd for UnionFindNode<Data> {
    fn partial_cmp(&self, other: &UnionFindNode<Data>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Data> Ord for UnionFindNode<Data> {
    fn cmp(&self, other: &UnionFindNode<Data>) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl<Data> Hash for UnionFindNode<Data> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

impl<Data> Clone for UnionFindNode<Data> {
    fn clone(&self) -> Self {
        UnionFindNode(self.0.clone())
    }
}

impl<Data: Default> Default for UnionFindNode<Data> {
    fn default() -> Self {
        UnionFindNode::new(Default::default())
    }
}

impl<Data> UnionFindNode<Data> {
    /// Creates a new singleton set with associated data.
    ///
    /// Initially this set is disjoint from all other sets, but can
    /// be joined with other sets using [`union`](#method.union).
    pub fn new(data: Data) -> Self {
        UnionFindNode(Rc::new(RefCell::new(Root {
            data: data,
            rank: 0,
        })))
    }

    /// Unions two sets, combining their data as specified.
    ///
    /// To determine the data associated with the set resulting from a
    /// union, we pass a closure `f`, which will be passed `self`’s data
    /// and `other`’s data. Then `f` must return a pair of the data to
    /// associate with the unioned set and any remaining value to return
    /// to the client.
    pub fn union_with<R, F>(&mut self, other: &mut Self, f: F) -> Option<R>
            where F: FnOnce(Data, Data) -> (Data, R) {

        let (a, rank_a) = self.find_with_rank();
        let (b, rank_b) = other.find_with_rank();

        if a == b {
            None
        } else if rank_a > rank_b {
            Some(b.set_parent_with(a, |b_data, a_data| f(a_data, b_data)))
        } else if rank_b > rank_a {
            Some(a.set_parent_with(b, f))
        } else {
            b.increment_rank();
            Some(a.set_parent_with(b, f))
        }
    }

    /// Unions two sets.
    ///
    /// Retains the data associated with an arbitrary set, returning the
    /// data of the other. Returns `None` if `self` and `other` are
    /// already elements of the same set.
    pub fn union(&mut self, other: &mut Self) -> Option<Data> {
        let (a, rank_a) = self.find_with_rank();
        let (b, rank_b) = other.find_with_rank();

        if a == b {
            None
        } else if rank_a > rank_b {
            Some(b.set_parent(a))
        } else if rank_b > rank_a {
            Some(a.set_parent(b))
        } else {
            b.increment_rank();
            Some(a.set_parent(b))
        }
    }

    // Can we do find iteratively?

    /// Finds a node representing the set of a given node.
    ///
    /// For two nodes in the same set, `find` returns the same node.
    pub fn find(&self) -> Self {
        match *self.0.borrow_mut() {
            Root { .. } => self.clone(),
            Link(ref mut parent) => {
                let root = parent.find();
                *parent = root.clone();
                root
            }
            Dummy => panic!("find: got dummy"),
        }
    }

    fn find_with_rank(&self) -> (Self, u8) {
        match *self.0.borrow_mut() {
            Root { rank, .. } => (self.clone(), rank),
            Link(ref mut parent) => {
                let (root, rank) = parent.find_with_rank();
                *parent = root.clone();
                (root, rank)
            }
            Dummy => panic!("find: got dummy"),
        }
    }

    /// Are the two nodes representatives of the same set?
    pub fn equiv(&self, other: &Self) -> bool {
        self.find() == other.find()
    }

    /// Replaces the data associated with the set.
    pub fn replace_data(&self, new: Data) -> Data {
        use std::mem::replace;
        self.with_data(|data| replace(data, new))
    }

    /// Returns a clone of the data associated with the set.
    pub fn clone_data(&self) -> Data
            where Data: Clone {
        self.with_data(|data| data.clone())
    }

    /// Allows modifying the data associated with a set.
    pub fn with_data<R, F>(&self, f: F) -> R
            where F: FnOnce(&mut Data) -> R {
        self.find().root_with_data(f)
    }

    // HELPERS

    fn root_with_data<R, F>(&self, f: F) -> R
            where F: FnOnce(&mut Data) -> R {

        match *self.0.borrow_mut() {
            Root { ref mut data, .. } => f(data),
            _ => panic!("with_data: non-root")
        }
    }

    fn increment_rank(&self) {
        match *self.0.borrow_mut() {
            Root { ref mut rank, .. } => {
                *rank += 1;
            }
            _ => panic!("increment_rank: non-root")
        }
    }

    fn set_parent(&self, new_parent: Self) -> Data {
        match mem::replace(&mut *self.0.borrow_mut(), Link(new_parent)) {
            Root { data, .. } => data,
            _ => panic!("set_parent: non-root"),
        }
    }

    // PRECONDITION:
    //  - self != parent
    //  - self and parent are both root nodes
    fn set_parent_with<R, F>(&self, parent: Self, f: F) -> R
            where F: FnOnce(Data, Data) -> (Data, R) {
        let mut guard_self = self.0.borrow_mut();
        let mut guard_parent = parent.0.borrow_mut();

        let contents_self = mem::replace(&mut *guard_self,
                                         Link(parent.clone()));
        let contents_parent = mem::replace(&mut *guard_parent, Dummy);

        match (contents_self, contents_parent) {
            (Root { data: data_self, .. },
             Root { data: data_parent, rank }) => {
                let (new_data, result) = f(data_self, data_parent);
                mem::replace(&mut *guard_parent, Root {
                    data: new_data,
                    rank: rank,
                });
                result
            }
            _ => panic!("set_parent_with: non-root"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union() {
        let mut uf0 = UnionFindNode::new(());
        let mut uf1 = UnionFindNode::new(());
        assert!(!uf0.equiv(&uf1));
        uf0.union(&mut uf1);
        assert!(uf0.equiv(&uf1));
    }

    #[test]
    fn unions() {
        let mut uf0 = UnionFindNode::new(());
        let mut uf1 = UnionFindNode::new(());
        let mut uf2 = UnionFindNode::new(());
        let mut uf3 = UnionFindNode::new(());
        let mut uf4 = UnionFindNode::new(());
        let mut uf5 = UnionFindNode::new(());
        let mut uf6 = UnionFindNode::new(());
        let mut uf7 = UnionFindNode::new(());

        uf0.union(&mut uf1);
        uf1.union(&mut uf2);
        uf4.union(&mut uf3);
        uf3.union(&mut uf2);
        assert!(uf0.equiv(&uf1));
        assert!(uf0.equiv(&uf2));
        assert!(uf0.equiv(&uf3));
        assert!(uf0.equiv(&uf4));
        assert!(!uf0.equiv(&uf5));

        uf3.union(&mut uf5);
        assert!(uf0.equiv(&uf5));

        uf7.union(&mut uf6);
        assert!(uf6.equiv(&uf7));
        assert!(!uf5.equiv(&uf7));

        uf0.union(&mut uf7);
        assert!(uf5.equiv(&uf7));
    }
}
