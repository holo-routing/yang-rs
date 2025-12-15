//
// Copyright (c) The yang-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

//! Rust bindings for the [libyang4] library.
//!
//! For raw FFI bindings for libyang4, see [libyang4-sys].
//!
//! [libyang4]: https://github.com/CESNET/libyang/tree/master
//! [libyang4-sys]: https://github.com/holo-routing/yang-rs/tree/master/libyang4-sys
//!
//! ## Design Goals
//! * Provide high-level bindings for libyang4 using idiomatic Rust
//! * Leverage Rust's ownership system to detect API misuse problems at compile
//!   time
//! * Automatic resource management
//! * Zero-cost abstractions
//!
//! ## Feature flags
//! By default, yang-rs uses pre-generated FFI bindings and uses dynamic
//! linking to load libyang4. The following feature flags, however, can be used
//! to change that behavior:
//! * **bundled**: instructs cargo to download and build libyang4 from the
//!   sources. The resulting objects are grouped into a static archive linked to
//!   this crate. This feature can be used when having a libyang4 dynamic link
//!   dependency isn't desirable.
//!   * Additional build requirements: *cc 1.0*, *cmake 0.1*, a C compiler and
//!     CMake.
//! * **use_bindgen**: generate new C FFI bindings dynamically instead of using
//!   the pre-generated ones. Useful when updating this crate to use newer
//!   libyang4 versions.
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
pub use libyang4_sys as ffi;
