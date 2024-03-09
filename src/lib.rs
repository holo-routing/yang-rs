//
// Copyright (c) The yang2-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

//! Rust bindings for the [libyang2] library.
//!
//! For raw FFI bindings for libyang2, see [libyang2-sys].
//!
//! [libyang2]: https://github.com/CESNET/libyang/tree/libyang2
//! [libyang2-sys]: https://github.com/holo-routing/yang2-rs/tree/master/libyang2-sys
//!
//! ## Design Goals
//! * Provide high-level bindings for libyang2 using idiomatic Rust
//! * Leverage Rust's ownership system to detect API misuse problems at compile
//!   time
//! * Automatic resource management
//! * Zero-cost abstractions
//!
//! ## Feature flags
//! By default, yang2-rs uses pre-generated FFI bindings and uses dynamic
//! linking to load libyang2. The following feature flags, however, can be used
//! to change that behavior:
//! * **bundled**: instructs cargo to download and build libyang2 from the
//!   sources. The resulting objects are grouped into a static archive linked to
//!   this crate. This feature can be used when having a libyang2 dynamic link
//!   dependency isn't desirable.
//!   * Additional build requirements: *cc 1.0*, *cmake 0.1*, a C compiler and
//!     CMake.
//! * **use_bindgen**: generate new C FFI bindings dynamically instead of using
//!   the pre-generated ones. Useful when updating this crate to use newer
//!   libyang2 versions.
//!   * Additional build requirements: *bindgen 0.55.0*
//!
//! ## Examples
//!
//! See <https://github.com/holo-routing/yang2-rs/tree/master/examples>

#![warn(rust_2018_idioms)]

mod error;

pub mod context;
pub mod data;
pub mod iter;
pub mod schema;
pub mod utils;

pub use crate::error::Error;

// Re-export the raw FFI bindings for convenience.
pub use libyang2_sys as ffi;
