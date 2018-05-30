//! Tree-based union-find with associated data.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::mem;

/// Pointer-based union-find representing disjoint sets with associated data.
///
/// This union-find implementation uses nodes to represent set elements
/// in a parent-pointer tree. Each set has associated with it an object
/// of type `Data`, which can be looked up and modified via any
/// representative of the set.
///
/// Construct a new singleton set with [`UnionFindNode::new`](#method.new).
///
/// # Examples
///
/// As an example, we perform first-order unification using
/// [`UnionFindNode`](struct.UnionFindNode.html)s to represent
/// unification variables.
///
/// ```
/// use disjoint_sets::UnionFindNode;
///
/// // A term is either a variable or a function symbol applied to some
/// // terms.
/// #[derive(Clone, Debug, PartialEq, Eq)]
/// enum Term {
///     Variable(String),
///     Constructor {
///         symbol: String,
///         params: Vec<Term>,
///     }
/// }
///
/// // Syntactic sugar for terms — write them LISP-style:
/// //
/// //   A             a variable
/// //
/// //   (f)           a nullary function symbol
/// //
/// //   (f A B (g))   function symbol applied to two variables and a
/// //                 function symbol
/// //
/// //   (arrow (tuple (vector A) (int)) A)
/// //                 type scheme of a polymorphic vector index function
/// //
/// macro_rules! term {
///     ( ( $symbol:ident $($args:tt)* ) )
///         =>
///     {
///         Term::Constructor {
///             symbol: stringify!($symbol).to_owned(),
///             params: vec![ $(term!($args)),* ],
///         }
///     };
///
///     ( $symbol:ident )
///         =>
///     {
///         Term::Variable(stringify!($symbol).to_owned())
///     };
/// }
///
/// // Internally we break terms down into variables about which we have
/// // no information, and variables that have unified with a function
/// // symbol applied to other variables.
/// #[derive(Clone, Debug)]
/// enum Term_ {
///     Indeterminate,
///     Fixed {
///         symbol: String,
///         params: Vec<Variable>,
///     },
/// }
/// type Variable = UnionFindNode<Term_>;
///
/// // To convert from external `Term`s to internal `Term_`s we use an
/// // environment mapping variable names to their internal
/// // representations as union-find nodes.
/// use std::collections::HashMap;
/// #[derive(Debug)]
/// struct Environment(HashMap<String, Variable>);
///
/// // The environment can get Rc-cycles in it (because we don’t do an
/// // occurs check, hence terms can be recursive). To avoid leaking, we
/// // need to clear the references out of it.
/// impl Drop for Environment {
///     fn drop(&mut self) {
///         for (_, v) in self.0.drain() {
///             v.replace_data(Term_::Indeterminate);
///         }
///     }
/// }
///
/// impl Term {
///     // Analyzes an external `Term`, converting it to internal
///     // `Term_`s and returning a variable mapped to it.
///     fn intern(self, env: &mut Environment) -> Variable {
///         match self {
///             Term::Variable(v) => {
///                 env.0.entry(v).or_insert_with(|| {
///                     UnionFindNode::new(Term_::Indeterminate)
///                 }).clone()
///             }
///
///             Term::Constructor { symbol, params } => {
///                 let params = params.into_iter()
///                     .map(|term| Term::intern(term, env))
///                     .collect::<Vec<_>>();
///                 UnionFindNode::new(Term_::Fixed {
///                     symbol: symbol,
///                     params: params,
///                 })
///             },
///         }
///     }
/// }
///
/// // A constraint is a collection of variables that need to unify,
/// // along with an environment mapping names to variables.
/// struct Constraint {
///     eqs: Vec<(Variable, Variable)>,
///     env: Environment,
/// }
///
/// impl Default for Constraint {
///     // Returns the empty (fully solved) constraint.
///     fn default() -> Self {
///         Constraint {
///             env: Environment(HashMap::new()),
///             eqs: Vec::new(),
///         }
///     }
/// }
///
/// impl Constraint {
///     // Creates a constraint that unifies two terms.
///     fn new(t1: Term, t2: Term) -> Self {
///         let mut new: Constraint = Default::default();
///         new.push(t1, t2);
///         new
///     }
///
///     // Adds two additional terms to unify.
///     fn push(&mut self, t1: Term, t2: Term) {
///         let v1 = t1.intern(&mut self.env);
///         let v2 = t2.intern(&mut self.env);
///         self.eqs.push((v1, v2))
///     }
///
///     // Performs a single unification step on a pair of variables.
///     // This may result in more equalities to add to the constraint.
///     fn unify(&mut self, mut v1: Variable, mut v2: Variable)
///              -> Result<(), String> {
///
///         match (v1.clone_data(), v2.clone_data()) {
///             (Term_::Indeterminate, _) => {
///                 v1.union_with(&mut v2, |_, t2| t2);
///                 Ok(())
///             },
///
///             (_, Term_::Indeterminate) => {
///                 v1.union_with(&mut v2, |t1, _| t1);
///                 Ok(())
///             },
///
///             (Term_::Fixed { symbol: symbol1, params: params1 },
///              Term_::Fixed { symbol: symbol2, params: params2 }) => {
///                 if symbol1 != symbol2 {
///                     let msg = format!(
///                         "Could not unify symbols: {} and {}",
///                         symbol1, symbol2);
///                     return Err(msg);
///                 }
///
///                 if params1.len() != params2.len() {
///                     let msg = format!(
///                         "Arity mismatch: {}: {} != {}",
///                         symbol1, params1.len(), params2.len());
///                     return Err(msg);
///                 }
///
///                 for (u1, u2) in params1.into_iter()
///                                        .zip(params2.into_iter()) {
///                     self.eqs.push((u1, u2));
///                 }
///
///                 v1.union(&mut v2);
///
///                 Ok(())
///             }
///         }
///     }
///
///     // Unifies equalities until there’s nothing left to do.
///     fn solve(mut self) -> Result<Environment, String> {
///         while let Some((v1, v2)) = self.eqs.pop() {
///             try!(self.unify(v1, v2));
///         }
///
///         Ok(self.env)
///     }
/// }
///
/// // Returns whether a pair of terms is unifiable.
/// fn unifiable(t1: Term, t2: Term) -> bool {
///     Constraint::new(t1, t2).solve().is_ok()
/// }
///
/// fn main() {
///     assert!(unifiable(term![ A ], term![ A ]));
///     assert!(unifiable(term![ A ], term![ B ]));
///
///     assert!(  unifiable(term![ (a) ], term![ (a) ]));
///     assert!(! unifiable(term![ (a) ], term![ (b) ]));
///
///     assert!(  unifiable(term![ (a A) ], term![ (a A) ]));
///     assert!(  unifiable(term![ (a A) ], term![ (a B) ]));
///     assert!(! unifiable(term![ (a A) ], term![ (b A) ]));
///     assert!(  unifiable(term![ (a A B) ], term![ (a B A) ]));
///     assert!(! unifiable(term![ (a A B C) ], term![ (a B A) ]));
///
///     assert!(  unifiable(term![ (a (b)) ], term![ (a (b)) ]));
///     assert!(! unifiable(term![ (a (b)) ], term![ (a (c)) ]));
///     assert!(  unifiable(term![ (a A A) ], term![ (a (b) (b)) ]));
///     assert!(! unifiable(term![ (a A A) ], term![ (a (b) (c)) ]));
///     assert!(  unifiable(term![ (a   (f) A   B   C  ) ],
///                         term![ (a   A   B   C   (f)) ]));
///     assert!(! unifiable(term![ (a   (f) A   B   C  ) ],
///                         term![ (a   A   B   C   (g)) ]));
///     assert!(  unifiable(term![ (a   (f) A   B   C  ) ],
///                         term![ (a   A   D   C   (g)) ]));
/// }
/// ```
#[derive(Default)]
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
    fn addr(&self) -> usize {
        &*self.0 as *const _ as usize
    }
}

impl<Data> Clone for UnionFindNode<Data> {
    fn clone(&self) -> Self {
        UnionFindNode(Rc::clone(&self.0))
    }
}

impl<Data> Debug for UnionFindNode<Data> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "UnionFindNode({:p})", self.0)
    }
}

impl<Data> PartialEq for UnionFindNode<Data> {
    fn eq(&self, other: &UnionFindNode<Data>) -> bool {
        self.addr() == other.addr()
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
        self.addr().cmp(&other.addr())
    }
}

impl<Data> Hash for UnionFindNode<Data> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr().hash(state)
    }
}

impl<Data: Default> Default for NodeImpl<Data> {
    fn default() -> Self {
        Self::new(Data::default())
    }
}

impl<Data> NodeImpl<Data> {
    fn new(data: Data) -> Self {
        Root {
            data: data,
            rank: 0,
        }
    }
}

impl<Data> UnionFindNode<Data> {
    /// Creates a new singleton set with associated data.
    ///
    /// Initially this set is disjoint from all other sets, but can
    /// be joined with other sets using [`union`](#method.union).
    pub fn new(data: Data) -> Self {
        UnionFindNode(Rc::new(RefCell::new(NodeImpl::new(data))))
    }

    /// Unions two sets, combining their data as specified.
    ///
    /// To determine the data associated with the set resulting from a
    /// union, we pass a closure `f`, which will be passed `self`’s data
    /// and `other`’s data (in that order). Then `f` must return the data to
    /// associate with the unioned set.
    pub fn union_with<F>(&mut self, other: &mut Self, f: F) -> bool
            where F: FnOnce(Data, Data) -> Data {

        let (a, rank_a) = self.find_with_rank();
        let (b, rank_b) = other.find_with_rank();

        if a == b {
            return false;
        }

        if rank_a > rank_b {
            b.set_parent_with(&a, |b_data, a_data| f(a_data, b_data))
        } else if rank_b > rank_a {
            a.set_parent_with(&b, f)
        } else {
            b.increment_rank();
            a.set_parent_with(&b, f)
        }

        true
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
    fn set_parent_with<F>(&self, parent: &Self, f: F)
            where F: FnOnce(Data, Data) -> Data {
        let mut guard_self = self.0.borrow_mut();
        let mut guard_parent = parent.0.borrow_mut();

        let contents_self = mem::replace(&mut *guard_self,
                                         Link(parent.clone()));
        let contents_parent = mem::replace(&mut *guard_parent, Dummy);

        match (contents_self, contents_parent) {
            (Root { data: data_self, .. },
             Root { data: data_parent, rank }) => {
                let new_data = f(data_self, data_parent);
                mem::replace(&mut *guard_parent, Root {
                    data: new_data,
                    rank: rank,
                });
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

    //
    // Unification example
    //

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Term {
        Variable(String),
        Constructor {
            symbol: String,
            params: Vec<Term>,
        }
    }

    macro_rules! term {
        ( ( $symbol:ident $($args:tt)* ) )
            =>
        {
            Term::Constructor {
                symbol: stringify!($symbol).to_owned(),
                params: vec![ $(term!($args)),* ],
            }
        };

        ( $symbol:ident )
            =>
        {
            Term::Variable(stringify!($symbol).to_owned())
        };
    }

    #[derive(Clone, Debug)]
    enum Term_ {
        Indeterminate,
        Fixed {
            symbol: String,
            params: Vec<Variable>,
        },
    }
    type Variable = UnionFindNode<Term_>;

    use std::collections::HashMap;
    #[derive(Debug)]
    struct Environment(HashMap<String, Variable>);

    // The environment can get Rc-cycles in it (because we don’t do an
    // occurs check, hence terms can be recursive). To avoid leaking, we
    // need to clear the references out of it.
    impl Drop for Environment {
        fn drop(&mut self) {
            for (_, v) in self.0.drain() {
                v.replace_data(Term_::Indeterminate);
            }
        }
    }

    impl Term {
        fn intern(self, env: &mut Environment) -> Variable {
            match self {
                Term::Variable(v) => {
                    env.0.entry(v).or_insert_with(|| {
                        UnionFindNode::new(Term_::Indeterminate)
                    }).clone()
                }

                Term::Constructor { symbol, params } => {
                    let params = params.into_iter()
                        .map(|term| Term::intern(term, env))
                        .collect::<Vec<_>>();
                    UnionFindNode::new(Term_::Fixed {
                        symbol: symbol,
                        params: params,
                    })
                },
            }
        }
    }

    struct Constraint {
        env: Environment,
        eqs: Vec<(Variable, Variable)>,
    }

    impl Default for Constraint {
        fn default() -> Self {
            Constraint {
                env: Environment(HashMap::new()),
                eqs: Vec::new(),
            }
        }
    }

    impl Constraint {
        fn new(t1: Term, t2: Term) -> Self {
            let mut new: Constraint = Default::default();
            new.push(t1, t2);
            new
        }

        fn push(&mut self, t1: Term, t2: Term) {
            let v1 = t1.intern(&mut self.env);
            let v2 = t2.intern(&mut self.env);
            self.eqs.push((v1, v2))
        }

        fn unify(&mut self, mut v1: Variable, mut v2: Variable)
                 -> Result<(), String> {

            match (v1.clone_data(), v2.clone_data()) {
                (Term_::Indeterminate, _) => {
                    v1.union_with(&mut v2, |_, t2| t2);
                    Ok(())
                },

                (_, Term_::Indeterminate) => {
                    v1.union_with(&mut v2, |t1, _| t1);
                    Ok(())
                },

                (Term_::Fixed { symbol: symbol1, params: params1 },
                 Term_::Fixed { symbol: symbol2, params: params2 }) => {
                    if symbol1 != symbol2 {
                        let msg = format!(
                            "Could not unify symbols: {} and {}",
                            symbol1, symbol2);
                        return Err(msg);
                    }

                    if params1.len() != params2.len() {
                        let msg = format!(
                            "Arity mismatch: {}: {} != {}",
                            symbol1, params1.len(), params2.len());
                        return Err(msg);
                    }

                    for (u1, u2) in params1.into_iter()
                                           .zip(params2.into_iter()) {
                        self.eqs.push((u1, u2));
                    }

                    v1.union(&mut v2);

                    Ok(())
                }
            }
        }

        fn solve(mut self) -> Result<Environment, String> {
            while let Some((v1, v2)) = self.eqs.pop() {
                try!(self.unify(v1, v2));
            }

            Ok(self.env)
        }
    }

    fn unifiable(t1: Term, t2: Term) -> bool {
        Constraint::new(t1, t2).solve().is_ok()
    }

    #[test]
    fn unify_vars() {
        assert!(unifiable(term![ A ], term![ A ]));
        assert!(unifiable(term![ A ], term![ B ]));
    }

    #[test]
    fn unify_symbols() {
        assert!(  unifiable(term![ (a) ], term![ (a) ]));
        assert!(! unifiable(term![ (a) ], term![ (b) ]));
    }

    #[test]
    fn unify_flat() {
        assert!(  unifiable(term![ (a A) ], term![ (a A) ]));
        assert!(  unifiable(term![ (a A) ], term![ (a B) ]));
        assert!(! unifiable(term![ (a A) ], term![ (b A) ]));
        assert!(  unifiable(term![ (a A B) ], term![ (a B A) ]));
        assert!(! unifiable(term![ (a A B C) ], term![ (a B A) ]));
    }

    #[test]
    fn unify_deeper() {
        assert!(  unifiable(term![ (a (b)) ], term![ (a (b)) ]));
        assert!(! unifiable(term![ (a (b)) ], term![ (a (c)) ]));
        assert!(  unifiable(term![ (a A A) ], term![ (a (b) (b)) ]));
        assert!(! unifiable(term![ (a A A) ], term![ (a (b) (c)) ]));
        assert!(  unifiable(term![ (a   (f) A   B   C  ) ],
                            term![ (a   A   B   C   (f)) ]));
        assert!(! unifiable(term![ (a   (f) A   B   C  ) ],
                            term![ (a   A   B   C   (g)) ]));
        assert!(  unifiable(term![ (a   (f) A   B   C  ) ],
                            term![ (a   A   D   C   (g)) ]));
    }
}
