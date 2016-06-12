# disjoint-sets: three union-find implementations

[![Build Status](https://travis-ci.org/tov/disjoint-sets-rs.svg?branch=master)](https://travis-ci.org/tov/disjoint-sets-rs)
[![Crates.io](https://img.shields.io/crates/v/disjoint-sets.svg?maxAge=2592000)](https://crates.io/crates/disjoint-sets)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE-APACHE)

This library provides three disjoint set data structures:

 - `UnionFind`: An array-based union-find
   where clients represent elements as small unsigned integers.
 - `UnionFindNode`: A tree-based
   union-find where each set can have associated ata, and where
   clients represent elements as opaque tree nodes.
 - `AUnionFind`: Like `UnionFind`, but `Sync` for sharing between
   threads.

All three perform rank-balanced path compression à la Tarjan, using
interior mutability.

## Usage

It’s [on crates.io](https://crates.io/crates/disjoint-sets), so it can be
used by adding `disjoint-sets` to the dependencies in your project’s
`Cargo.toml`:

```toml
[dependencies]
disjoint-sets = "*"
```
