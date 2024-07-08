//
// Copyright (c) The yang-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

//! YANG iterators.

use crate::context::Context;
use crate::data::Metadata;
use crate::schema::SchemaModule;
use crate::utils::Binding;
use libyang3_sys as ffi;

/// Common methods used by multiple data and schema node iterators.
#[doc(hidden)]
pub trait NodeIterable<'a>: Sized + Clone + PartialEq + Binding<'a> {
    /// Returns the parent node.
    fn parent(&self) -> Option<Self>;

    /// Returns the next sibling node.
    fn next_sibling(&self) -> Option<Self>;

    /// Returns the fist child none.
    fn first_child(&self) -> Option<Self>;
}

/// An iterator over the sibings of a node.
#[derive(Debug)]
pub struct Siblings<'a, T>
where
    T: NodeIterable<'a>,
{
    next: Option<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

/// An iterator over the ancestors of a node.
#[derive(Debug)]
pub struct Ancestors<'a, T>
where
    T: NodeIterable<'a>,
{
    next: Option<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

/// An iterator over all elements in a tree (depth-first search algorithm).
///
/// When traversing over schema trees, note that _actions_ and _notifications_
/// are ignored.
#[derive(Debug)]
pub struct Traverse<'a, T>
where
    T: NodeIterable<'a>,
{
    start: T,
    next: Option<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

/// An iterator over a set of nodes.
///
/// This is a safe wrapper around ffi::ly_set.
#[derive(Debug)]
pub struct Set<'a, T>
where
    T: NodeIterable<'a>,
{
    container: &'a T::Container,
    slice: &'a [*mut T::CType],
}

/// An iterator over an array of nodes or substatements.
///
/// This is a safe wrapper around libyang3's
/// [sized arrays](https://netopeer.liberouter.org/doc/libyang/master/html/howto_structures.html).
#[derive(Debug)]
pub struct Array<'a, S: Binding<'a>> {
    context: &'a Context,
    raw: *mut S::CType,
    ptr_size: usize,
    count: usize,
}

/// An iterator over a list of schema modules.
#[derive(Debug)]
pub struct SchemaModules<'a> {
    context: &'a Context,
    index: u32,
}

/// An iterator over a list of metadata.
#[derive(Debug)]
pub struct MetadataList<'a> {
    next: Option<Metadata<'a>>,
}

// ===== impl Siblings =====

impl<'a, T> Siblings<'a, T>
where
    T: NodeIterable<'a>,
{
    pub fn new(next: Option<T>) -> Siblings<'a, T> {
        Siblings {
            next,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for Siblings<'a, T>
where
    T: NodeIterable<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let ret = self.next.clone();
        if let Some(next) = &self.next {
            self.next = next.next_sibling();
        }
        ret
    }
}

// ===== impl Ancestors =====

impl<'a, T> Ancestors<'a, T>
where
    T: NodeIterable<'a>,
{
    pub fn new(next: Option<T>) -> Ancestors<'a, T> {
        Ancestors {
            next,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for Ancestors<'a, T>
where
    T: NodeIterable<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let node = self.next.clone();
        if let Some(next) = &self.next {
            self.next = next.parent();
        }
        node
    }
}

// ===== impl Traverse =====

impl<'a, T> Traverse<'a, T>
where
    T: NodeIterable<'a>,
{
    pub fn new(start: T) -> Traverse<'a, T> {
        let next = start.clone();

        Traverse {
            start,
            next: Some(next),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for Traverse<'a, T>
where
    T: NodeIterable<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let ret = self.next.clone();

        if let Some(elem) = &mut self.next {
            // Select element for the next run - children first.
            let mut next_elem = elem.first_child();
            if next_elem.is_none() {
                // Check end condition.
                if *elem == self.start {
                    self.next = None;
                    return ret;
                }

                // No children, try siblings.
                next_elem = elem.next_sibling();
            }

            while next_elem.is_none() {
                // Parent is already processed, go to its sibling.
                *elem = elem.parent().unwrap();

                // Check end condition.
                if *elem == self.start {
                    self.next = None;
                    return ret;
                }
                next_elem = elem.next_sibling();
            }

            *elem = next_elem.unwrap();
        }

        ret
    }
}

// ===== impl Set =====

impl<'a, T> Set<'a, T>
where
    T: NodeIterable<'a>,
{
    pub fn new(
        container: &'a T::Container,
        slice: &'a [*mut T::CType],
    ) -> Set<'a, T> {
        Set { container, slice }
    }
}

impl<'a, T> Iterator for Set<'a, T>
where
    T: NodeIterable<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if !self.slice.is_empty() {
            let dnode =
                Some(unsafe { T::from_raw(self.container, self.slice[0]) });
            self.slice = &self.slice[1..];
            dnode
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slice.len()))
    }
}

unsafe impl<'a, T> Send for Set<'a, T> where T: NodeIterable<'a> {}
unsafe impl<'a, T> Sync for Set<'a, T> where T: NodeIterable<'a> {}

// ===== impl Array =====

impl<'a, S> Array<'a, S>
where
    S: Binding<'a>,
{
    pub fn new(
        context: &'a Context,
        raw: *mut S::CType,
        ptr_size: usize,
    ) -> Array<'a, S> {
        // Get the number of records in the array (equivalent to
        // LY_ARRAY_COUNT).
        let count = if raw.is_null() {
            0
        } else {
            unsafe { (raw as *const usize).offset(-1).read() }
        };

        Array {
            context,
            raw,
            ptr_size,
            count,
        }
    }
}

impl<'a, S> Iterator for Array<'a, S>
where
    S: Binding<'a, Container = Context>,
{
    type Item = S;

    fn next(&mut self) -> Option<S> {
        if self.count > 0 {
            let next = unsafe { S::from_raw_opt(self.context, self.raw) };
            self.count -= 1;
            self.raw = (self.raw as usize + self.ptr_size) as *mut S::CType;
            next
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.count))
    }
}

unsafe impl<'a, S> Send for Array<'a, S> where S: NodeIterable<'a> {}
unsafe impl<'a, S> Sync for Array<'a, S> where S: NodeIterable<'a> {}

// ===== impl SchemaModules =====

impl<'a> SchemaModules<'a> {
    pub fn new(context: &'a Context, skip_internal: bool) -> SchemaModules<'a> {
        let index = if skip_internal {
            context.internal_module_count()
        } else {
            0
        };
        SchemaModules { context, index }
    }
}

impl<'a> Iterator for SchemaModules<'a> {
    type Item = SchemaModule<'a>;

    fn next(&mut self) -> Option<SchemaModule<'a>> {
        let rmodule = unsafe {
            ffi::ly_ctx_get_module_iter(self.context.raw, &mut self.index)
        };
        unsafe { SchemaModule::from_raw_opt(self.context, rmodule as *mut _) }
    }
}

// ===== impl MetadataList =====

impl MetadataList<'_> {
    pub fn new(next: Option<Metadata<'_>>) -> MetadataList<'_> {
        MetadataList { next }
    }
}

impl<'a> Iterator for MetadataList<'a> {
    type Item = Metadata<'a>;

    fn next(&mut self) -> Option<Metadata<'a>> {
        let meta = self.next.clone();
        if let Some(next) = &self.next {
            self.next = next.next();
        }
        meta
    }
}
