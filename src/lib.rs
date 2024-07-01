//! # RocStr
//!
//! ## Overview
//!
//! Rust OuiCloud Str
//!
//! An immutable fixed capacity stack based generic copy string.
//!
//! The RocStr is a string backed by a fixed size array.
//!
//! It keeps track of its length, and is parameterized by SIZE for the maximum capacity.
//!
//! SIZE is of type usize but is range limited to u32::MAX; attempting to create RocStr with larger capacity will panic.
//!
//! ## Usage
//!
//! ### Basic usage
//!
//! ```bash
//! cargo install rocstr
//! ```
//!
//! ```rust
//! # use rocstr::RocStr;
//!
//! #[derive(Copy, Clone)]
//! pub struct Customer {
//!     id: u32,
//!     first_name: RocStr<64>,
//!     last_name: RocStr<64>,
//! }
//!
//! let alice = Customer {id: 1, first_name: "Alice".into(), last_name: "Adams".into()};
//! let bob = Customer {id: 2, first_name: "Bob".into(), last_name: "Bennett".into()};
//!
//! let alice_name = alice.first_name + " " + alice.last_name;
//! let bob_name = bob.first_name + " " + bob.last_name;
//!
//! assert_eq!(alice_name, "Alice Adams");
//! assert_eq!(bob_name, "Bob Bennett");
//! ```
//!
//! ## Motivation
//!
//! Yet another copy string type for Rust.
//!
//! But, this one is particularly shaped to ease async programming :
//! - it is owned which means no lifetime management, unlike Rust standard `&str`
//! - it implements `Copy` trait, unlike Rust standard `String`, which means fast and implicit copy
//! - it exposed an immutable API, no interior mutability, which means thread safety
//!
//! More over, it was designed to cover web full stack programming range, from full featured back-end server to WASM front-end.
//!
//! | Library                  | owned | impl Copy | immut. API | no std | no unsafe | Note                             |
//! |--------------------------|:-----:|:---------:|:----------:|:------:|:---------:|----------------------------------|
//! | core::str                | ❌    | ❌         | ❌         | ✅     | ➖        | core immutable string             |
//! | std::String              | ✅    | ❌         | ❌         | ❌     | ➖        | std string                        |
//! | imstr::ImString          | ✅    | ❌         | ❌         | ❌     | ❌        | use `Arc<String>` under the hood  |
//! | smol_str::SmolStr        | ✅    | ❌         | ❌         | ✅     | ❌        | rust-analyzer string              |
//! | bytestring::ByteString   | ✅    | ❌         | ❌         | ✅     | ❌        | actix string                      |
//! | flexstr::FlexStr         | ✅    | ❌         | ❌         | ✅     | ❌        |                                   |
//! | copstr::Str              | ✅    | ✅         | ❌         | ❌     | ❌        |                                   |
//! | copystr::sXX             | ✅    | ✅         | ❌         | ❌     | ✅        | old impl before const generic     |
//! | arraystring::ArrayString | ✅    | ✅         | ❌         | ✅     | ❌        | old impl before const generic     |
//! | tinystr::TinyAsciiStr    | ✅    | ✅         | ❌         | ✅     | ❌        | ascii only                        |
//! | arrayvec::ArrayString    | ✅    | ✅         | ❌         | ✅     | ❌        | unfortunately, it uses unsafe     |
//! | rocstr::RocStr           | ✅    | ✅         | ✅         | ✅     | ✅        | this crate                        |
//!
//! ## Use cases
//!
//! The target use case is web full stack programming :
//! - mapping database strings with bound size (char(SIZE), varchar(SIZE), etc...)
//! - easing async functional programming (owned, impl Copy, immutability)
//!
//! ## Supported Rust Versions
//!
//! The current MSRV is 1.60.
//!
//! ## Safety
//!
//! This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.
//!
//! ## Crate Features
//!
//! RocStr is built with these features enabled by default:
//! - std enables functionality dependent on the std lib
//!
//! Optionally, the following dependencies can be enabled:
//! - serde enables serde Serialize/Deserialize support
//! - postgres enables PostgreSql type support
//!
//! RocStr supports no_std mode (enabled via default-features = false)
//!
//! ## License
//!
//! RocStr is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
//!
//! See LICENSE-APACHE and LICENSE-MIT, and COPYRIGHT for details.

#![forbid(unsafe_code)]
#![no_std]

pub mod rocerr;
pub mod rocstr;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "serde")]
pub mod serialize;

pub use crate::rocerr::InsufficientCapacity;
pub use crate::rocstr::RocStr;
