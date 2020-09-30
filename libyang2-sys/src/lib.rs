#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Silence "128-bit integers don't currently have a known stable ABI" warnings
#![allow(improper_ctypes)]
// Silence "constants have by default a `'static` lifetime" clippy warnings
#![allow(clippy::redundant_static_lifetimes)]

include!(concat!(env!("OUT_DIR"), "/libyang2.rs"));
