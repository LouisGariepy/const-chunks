# `const-chunks`

<div align="center">
  <!-- Version -->
  <a href="https://crates.io/crates/const-chunks">
    <img src="https://img.shields.io/crates/v/const-chunks.svg?style=flat-square"
    alt="Crates.io version" />
  </a>

  <!-- Docs -->
  <a href="https://docs.rs/const-chunks/latest/const_chunks/">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/const-chunks?style=flat-square">
  </a>
  
  <!-- Dependencies -->
  <a href="https://deps.rs/repo/github/LouisGariepy/const-chunks">
    <img src="https://deps.rs/repo/github/LouisGariepy/const-chunks/status.svg?style=flat-square"
    alt="Crates.io version" />
  </a>

  <!-- no_std -->
  <picture>
    <img alt="no_std compatible" src="https://img.shields.io/badge/no__std-compatible-light_green?style=flat-square"/>
  </picture>
 
  <!-- License -->
  <a href="https://github.com/LouisGariepy/const-chunks#License">
    <img src="https://img.shields.io/badge/License-APACHE--2.0%2FMIT-blue?style=flat-square" alt="License">
  </a>
</div>

<br/>

This crate provides a `#![no_std]`-compatible extension trait that lets you chunk iterators into constant-length arrays using `const` generics.

See the [docs](https://docs.rs/const-chunks/latest/const_chunks/) for more info.

```rust
use const_chunks::IteratorConstChunks;

let mut iter = vec![1, 2, 3, 4, 5].into_iter().const_chunks::<2>();
assert_eq!(iter.next(), Some([1,2]));
assert_eq!(iter.next(), Some([3,4]));
assert_eq!(iter.next(), None);

let mut remainder = iter.into_remainder().unwrap();
assert_eq!(remainder.next(), Some(5));
assert_eq!(remainder.next(), None);
```

## Safety

This crate uses unsafe to manipulate uninitialized memory.

To prevent undefined behavior, the code runs MIRI in CI. It's also well-documented and easy to audit.

Nevertheless, you should still consider this fact if you're trying to minimize unsafe dependencies.

## MSRV

This crate requires `rustc` version 1.65 or newer.

This crate's MSRV is enforced through the manifest's `rust-version` key.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
