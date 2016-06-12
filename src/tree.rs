//! Tree-based union-find with associated data.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::mem;

/// Union-find with associated data.
///
/// This union-find implementation uses nodes to represent set elements
/// in a parent-pointer tree. Each set has associated with it an object
/// of type `Data`, which can be looked up and modified via any
/// representative of the set.
///
/// Construct a new singleton set with [`Node::new`](#method.new).
pub struct Node<Data = ()>(Rc<RefCell<NodeImpl<Data>>>);

enum NodeImpl<Data> {
    Root {
        data: Data,
        rank: u8,
    },
    Link(Node<Data>),
}

use self::NodeImpl::*;

impl<Data> Node<Data> {
    fn id(&self) -> usize {
        &*self.0 as *const _ as usize
    }
}

impl<Data> PartialEq for Node<Data> {
    fn eq(&self, other: &Node<Data>) -> bool {
        self.id() == other.id()
    }
}

impl<Data> Eq for Node<Data> { }

impl<Data> PartialOrd for Node<Data> {
    fn partial_cmp(&self, other: &Node<Data>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Data> Ord for Node<Data> {
    fn cmp(&self, other: &Node<Data>) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl<Data> Hash for Node<Data> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

impl<Data> Clone for Node<Data> {
    fn clone(&self) -> Self {
        Node(self.0.clone())
    }
}

impl<Data: Default> Default for Node<Data> {
    fn default() -> Self {
        Node::new(Default::default())
    }
}

impl<Data> Node<Data> {
    /// Creates a new singleton set with associated data.
    ///
    /// Initially this set is disjoint from all other sets, but can
    /// be joined with other sets using [`union`](#method.union).
    pub fn new(data: Data) -> Self {
        Node(Rc::new(RefCell::new(Root {
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
            return None
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

        match &mut *self.0.borrow_mut() {
            &mut Root { ref mut data, .. } => f(data),
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
            Link(_) => panic!("set_parent: non-root"),
        }
    }

    fn set_parent_with<R, F>(&self, parent: Self, f: F) -> R
            where F: FnOnce(Data, Data) -> (Data, R) {
        use std::ptr;

        let mut guard_self = self.0.borrow_mut();

        let result = match (&mut *guard_self, &mut *parent.0.borrow_mut()) {
            (&mut Root { data: ref mut data_self, .. },
             &mut Root { data: ref mut data_parent, .. }) => {
                unsafe {
                    let (new_data, result) = f(ptr::read(data_self),
                                               ptr::read(data_parent));
                    ptr::write(data_parent, new_data);
                    result
                }
            }
            _ => panic!("set_parent_with: non-root"),
        };

        unsafe { ptr::write(&mut *guard_self, Link(parent)); }
        result
    }
}
