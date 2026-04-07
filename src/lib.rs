//
// Copyright (c) The yang-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

//! Rust bindings for the [libyang5] library.
//!
//! For raw FFI bindings for libyang5, see [libyang5-sys].
//!
//! [libyang5]: https://github.com/CESNET/libyang/tree/master
//! [libyang5-sys]: https://github.com/holo-routing/yang-rs/tree/master/libyang5-sys
//!
//! ## Design Goals
//! * Provide high-level bindings for libyang5 using idiomatic Rust
//! * Leverage Rust's ownership system to detect API misuse problems at compile
//!   time
//! * Automatic resource management
//! * Zero-cost abstractions
//!
//! ## Feature flags
//! By default, yang-rs uses pre-generated FFI bindings and uses dynamic
//! linking to load libyang5. The following feature flags, however, can be used
//! to change that behavior:
//! * **bundled**: instructs cargo to download and build libyang5 from the
//!   sources. The resulting objects are grouped into a static archive linked to
//!   this crate. This feature can be used when having a libyang5 dynamic link
//!   dependency isn't desirable.
//!   * Additional build requirements: *cc 1.0*, *cmake 0.1*, a C compiler and
//!     CMake.
//! * **use_bindgen**: generate new C FFI bindings dynamically instead of using
//!   the pre-generated ones. Useful when updating this crate to use newer
//!   libyang5 versions.
//!   * Additional build requirements: *bindgen 0.68.0*
//!
//! ## Examples
//!
//! See <https://github.com/holo-routing/yang-rs/tree/master/examples>

mod error;

pub mod context;
pub mod data;
pub mod iter;
pub mod logging;
pub mod schema;
pub mod utils;

pub use crate::error::Error;

// Re-export the raw FFI bindings for convenience.
pub use libyang5_sys as ffi;
