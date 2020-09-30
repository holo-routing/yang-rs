//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

use std::ffi::CStr;

/// Convert C String to string slice.
pub(crate) fn char_ptr_to_string(
    c_str: *const ::std::os::raw::c_char,
) -> String {
    unsafe { CStr::from_ptr(c_str).to_string_lossy().into_owned() }
}

/// Convert C String to optional string slice.
pub(crate) fn char_ptr_to_opt_string(
    c_str: *const ::std::os::raw::c_char,
) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        Some(char_ptr_to_string(c_str))
    }
}

/// Convert C String to owned string.
pub(crate) fn char_ptr_to_str<'a>(
    c_str: *const ::std::os::raw::c_char,
) -> &'a str {
    unsafe { CStr::from_ptr(c_str).to_str().unwrap() }
}

/// Convert C String to optional owned string.
pub(crate) fn char_ptr_to_opt_str<'a>(
    c_str: *const ::std::os::raw::c_char,
) -> Option<&'a str> {
    if c_str.is_null() {
        None
    } else {
        Some(char_ptr_to_str(c_str))
    }
}

/// A trait implemented by all types that can be created from a raw C pointer
/// and a generic container type.
#[doc(hidden)]
pub trait Binding<'a>
where
    Self: Sized,
    <Self as Binding<'a>>::Container: 'a,
{
    type CType;
    type Container;

    fn from_raw(container: &'a Self::Container, raw: *mut Self::CType) -> Self;

    fn from_raw_opt(
        container: &'a Self::Container,
        raw: *mut Self::CType,
    ) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self::from_raw(&container, raw))
        }
    }
}
