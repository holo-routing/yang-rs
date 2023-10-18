//
// Copyright (c) The yang2-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

use crate::context::Context;
use crate::utils::*;
use libyang2_sys as ffi;

/// A convenience wrapper around `Result` for `yang2::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Enum listing possible errors from yang2-rs.
#[derive(Debug, Eq, PartialEq)]
pub struct Error {
    pub errcode: ffi::LY_ERR::Type,
    pub msg: Option<String>,
    pub path: Option<String>,
    pub apptag: Option<String>,
}

impl Error {
    pub fn new(ctx: &Context) -> Error {
        let errcode = unsafe { ffi::ly_errcode(ctx.raw) };
        let msg = unsafe { ffi::ly_errmsg(ctx.raw) };
        let msg = char_ptr_to_opt_string(msg);
        let path = unsafe { ffi::ly_errpath(ctx.raw) };
        let path = char_ptr_to_opt_string(path);
        let apptag = unsafe { ffi::ly_errapptag(ctx.raw) };
        let apptag = char_ptr_to_opt_string(apptag);

        Self {
            errcode,
            msg,
            path,
            apptag,
        }
    }
}

impl std::fmt::Display for Error {
    // Print only the base error message by default.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "{}", msg)
        } else {
            write!(f, "Unknown error: {}", self.errcode)
        }
    }
}

impl std::error::Error for Error {}
