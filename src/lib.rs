//! Implementations of Tarjan’s union-find data structure for disjoint
//! sets.
//!
//! The two variants are:
//!
//!  - [`UnionFind`](struct.UnionFind.html): An array-based union-find
//!    where clients represent elements as small unsigned integers.
//!  - [`UnionFindNode`](struct.UnionFindNode.html): A tree-based
//!    union-find where each set can have associated ata, and where
//!    clients represent elements as opaque tree nodes.
//!
//! Both data structures perform rank-balanced path compression
//! using interior mutability.

mod traits;
mod array;
mod tree;

pub use traits::*;
pub use array::*;
pub use tree::*;

