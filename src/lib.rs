//! Three union-find implementations
//!
//! The variants are:
//!
//!  - [`UnionFind`](struct.UnionFind.html): An array-based union-find
//!    where clients represent elements as small unsigned integers.
//!  - [`UnionFindNode`](struct.UnionFindNode.html): A tree-based
//!    union-find where each set can have associated ata, and where
//!    clients represent elements as opaque tree nodes.
//!  - [`AUnionFind`](struct.AUnionFind.html): Like `UnionFind`, but
//!    it’s `Sync` for sharing between threads.
//!
//! All three perform rank-balanced path compression à la Tarjan,
//! using interior mutability.

mod traits;
mod array;
mod tree;
mod async;

pub use traits::*;
pub use array::*;
pub use tree::*;
pub use async::*;

