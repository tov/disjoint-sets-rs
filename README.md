# disjoint-sets-rs: Implementations of Tarjan’s Union-Find

[![Build Status](https://travis-ci.org/tov/disjoint-sets-rs.svg?branch=master)](https://travis-ci.org/tov/disjoint-sets-rs)

This library provides two disjoint set data structures:

 - [`UnionFind`](struct.UnionFind.html): An array-based union-find
   where clients represent elements as small unsigned integers.
 - [`UnionFindNode`](struct.UnionFindNode.html): A tree-based
   union-find where each set can have associated ata, and where
   clients represent elements as opaque tree nodes.

Both perform rank-balanced path compression using interior mutability.
