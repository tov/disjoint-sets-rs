#![doc(html_root_url = "https://docs.rs/disjoint-sets/0.4.2")]
//! Three union-find implementations.
//!
//! The variants are:
//!
//! |           | structure | element type | data? | concurrent? |
//! | :-------- | :-------- | :----------- | :---- | :---------- |
//! | [`UnionFind`](struct.UnionFind.html) | vector | small integer | no | no |
//! | [`UnionFindNode`](struct.UnionFindNode.html) | tree | tree node | yes | no |
//! | [`AUnionFind`](struct.AUnionFind.html) | array | `usize` | no | yes |
//!
//! All three perform rank-balanced path halving à la Tarjan,
//! using interior mutability.
//!
//! # Usage
//!
//! It’s [on crates.io](https://crates.io/crates/disjoint-sets), so add
//! this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! disjoint-sets = "0.4.2"
//! ```
//!
//! And add this to your crate root:
//!
//! ```
//! extern crate disjoint_sets;
//! ```
//!
//! This crate supports Rust version 1.21 and later.
//!
//! # Examples
//!
//! Kruskal’s algorithm to find the minimum spanning tree of a graph:
//!
//! ```
//! use disjoint_sets::UnionFind;
//! use std::collections::HashSet;
//!
//! type Node = usize;
//! type Weight = usize;
//!
//! struct Neighbor {
//!     dst: Node,
//!     weight: Weight,
//! }
//!
//! type Graph = Vec<Vec<Neighbor>>;
//!
//! fn edges_by_weight(graph: &Graph) -> Vec<(Node, Node, Weight)> {
//!     let mut edges = vec![];
//!
//!     for (src, dsts) in graph.iter().enumerate() {
//!         for edge in dsts {
//!             edges.push((src, edge.dst, edge.weight));
//!         }
//!     }
//!
//!     edges.sort_by_key(|&(_, _, weight)| weight);
//!     edges
//! }
//!
//! fn mst(graph: &Graph) -> HashSet<(Node, Node)> {
//!     let mut result = HashSet::new();
//!     let mut uf = UnionFind::new(graph.len());
//!
//!     for (src, dst, _) in edges_by_weight(graph) {
//!         if uf.union(src, dst) {
//!             result.insert((src, dst));
//!         }
//!     }
//!
//!     result
//! }
//!
//! fn main() {
//!     // Graph to use:
//!     //
//!     //  0 ------ 1 ------ 2
//!     //  |    6   |    5   |
//!     //  | 8      | 1      | 4
//!     //  |        |        |
//!     //  3 ------ 4 ------ 5
//!     //  |    7   |    2   |
//!     //  | 3      | 12     | 11
//!     //  |        |        |
//!     //  6 ------ 7 ------ 8
//!     //       9        10
//!     let graph = vec![
//!         // Node 0
//!         vec![ Neighbor { dst: 1, weight: 6 },
//!               Neighbor { dst: 3, weight: 8 }, ],
//!         // Node 1
//!         vec![ Neighbor { dst: 2, weight: 5 },
//!               Neighbor { dst: 4, weight: 1 }, ],
//!         // Node 2
//!         vec![ Neighbor { dst: 5, weight: 4 }, ],
//!         // Node 3
//!         vec![ Neighbor { dst: 4, weight: 7 },
//!               Neighbor { dst: 6, weight: 3 }, ],
//!         // Node 4
//!         vec![ Neighbor { dst: 5, weight: 2 },
//!               Neighbor { dst: 7, weight: 12 }, ],
//!         // Node 5
//!         vec![ Neighbor { dst: 8, weight: 11 }, ],
//!         // Node 6
//!         vec![ Neighbor { dst: 7, weight: 9 }, ],
//!         // Node 7
//!         vec![ Neighbor { dst: 8, weight: 10 }, ],
//!         // Node 8
//!         vec![ ],
//!     ];
//!
//!     assert_eq! {
//!         mst(&graph),
//!         vec![ (1, 4), (4, 5), (3, 6), (2, 5),
//!               (0, 1), (3, 4), (6, 7), (7, 8), ]
//!              .into_iter().collect::<HashSet<_>>()
//!     };
//! }
//! ```

#![warn(missing_docs)]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod traits;
mod array;
mod tree;
mod async;

pub use traits::*;
pub use array::*;
pub use tree::*;
pub use async::*;

