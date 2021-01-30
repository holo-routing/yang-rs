//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! Rust bindings for the [libyang2] library.
//!
//! For raw FFI bindings for libyang2, see [libyang2-sys].
//!
//! [libyang2]: https://github.com/CESNET/libyang/tree/libyang2
//! [libyang2-sys]: https://github.com/rwestphal/yang2-rs/tree/master/libyang2-sys
//!
//! ## Design Goals
//! * Provide high-level bindings for libyang2 using idiomatic Rust
//! * Leverage Rust's ownership system to detect API misuse problems at compile
//!   time
//! * Automatic resource management
//! * Zero-cost abstractions
//!
//! ## Examples
//!
//! See <https://github.com/rwestphal/yang2-rs/tree/master/examples>

mod error;
mod utils;

pub mod context;
pub mod data;
pub mod iter;
pub mod schema;

pub use crate::error::Error;
