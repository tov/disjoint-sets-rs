# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

## [Unreleased]

### Changed
- `AUnionFind` now allows no more than 2<sup>58</sup> objects on 64-bit
platforms (no more than 2<sup>27</sup> objects on 32-bit platforms).

### Added
- Random testing of `AUnionFind` using `quickcheck`.

## [0.4.2] - 2018-05-30

### Fixed
- Version metadata.

## [0.4.1] - 2018-05-30

### Added
- `#![doc(html_root_url = ...)]` annotation.

## [0.4.0] - 2018-05-30

### Added
- Serde support for `UnionFind` and `AUnionFind`. Pass feature
  flag `"serde"` to Cargo to enable `Serializable` and `Deserializable`
  impls for both types.
  
### Changed
- Renamed `UnionFind::as_vec` and `AUnionFind::as_vec` to `to_vec`, to
  better reflect the cost of the methods.
  
### Fixed
- `AUnionFind::force` now correctly forces all laziness.
- `AUnionFind::equiv` now checks that nothing has become equivalent before 
  returning `false`.

