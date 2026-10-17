//
// Copyright (c) The yang-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

use libyang5_sys as ffi;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Convert C String to owned string.
pub(crate) fn char_ptr_to_string(c_str: *const c_char, free: bool) -> String {
    let string =
        unsafe { CStr::from_ptr(c_str).to_string_lossy().into_owned() };
    if free {
        unsafe { ffi::free(c_str as *mut std::ffi::c_void) };
    }
    string
}

/// Convert C String to optional owned string.
pub(crate) fn char_ptr_to_opt_string(
    c_str: *const c_char,
    free: bool,
) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        Some(char_ptr_to_string(c_str, free))
    }
}

/// Convert C String to string slice.
pub(crate) fn char_ptr_to_str<'a>(c_str: *const c_char) -> &'a str {
    unsafe { CStr::from_ptr(c_str).to_str().unwrap() }
}

/// Convert C String to optional string slice.
pub(crate) fn char_ptr_to_opt_str<'a>(c_str: *const c_char) -> Option<&'a str> {
    if c_str.is_null() {
        None
    } else {
        Some(char_ptr_to_str(c_str))
    }
}

/// Returns the number of items of a libyang sized array (LY_ARRAY), which
/// may be null (equivalent to LY_ARRAY_COUNT).
///
/// The number is stored in the LY_ARRAY_COUNT_TYPE preceding the array, which
/// is a uint64_t regardless of the pointer width.
pub(crate) unsafe fn ly_array_count<T>(array: *const T) -> usize {
    if array.is_null() {
        return 0;
    }
    unsafe { (array as *const u64).offset(-1).read() as usize }
}

/// Returns the items of a libyang sized array (LY_ARRAY), which may be null.
pub(crate) unsafe fn ly_array<'a, T>(array: *const T) -> &'a [T] {
    if array.is_null() {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(array, ly_array_count(array)) }
}

/// A trait implemented by all types that can be created from a raw C pointer
/// and a generic container type.
pub unsafe trait Binding<'a>
where
    Self: Sized,
    <Self as Binding<'a>>::Container: 'a,
{
    type CType;
    type Container;

    unsafe fn from_raw(
        container: &'a Self::Container,
        raw: *mut Self::CType,
    ) -> Self;

    unsafe fn from_raw_opt(
        container: &'a Self::Container,
        raw: *mut Self::CType,
    ) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self::from_raw(container, raw))
        }
    }
}
