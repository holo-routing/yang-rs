//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! YANG iterators.

use crate::context::Context;
use crate::data::Metadata;
use crate::schema::SchemaModule;
use crate::utils::Binding;
use libyang2_sys as ffi;

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
    _marker: std::marker::PhantomData<&'a T>,
}

/// An iterator over an array of nodes or substatements.
///
/// This is a safe wrapper around libyang2's
/// [sized arrays](https://netopeer.liberouter.org/doc/libyang/libyang2/html/howto_structures.html#sizedarrays).
#[derive(Debug)]
pub struct Array<'a, S: Binding<'a>> {
    context: &'a Context,
    raw: *mut S::CType,
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

        if let Some(next) = &mut self.next {
            // Select element for the next run - children first.
            *next = match next.first_child() {
                Some(child) => child,
                None => {
                    // No children.
                    if *next == self.start {
                        self.next = None;
                        return ret;
                    }

                    // Try siblings.
                    loop {
                        match next.next_sibling() {
                            Some(iter) => break iter,
                            None => {
                                // Parent is already processed, go to its
                                // sibling.
                                *next = next.parent().unwrap();

                                // If no siblings, go back through parents.
                                if next.parent() != self.start.parent() {
                                    continue;
                                }

                                // We are done, no next element to process.
                                self.next = None;
                                return ret;
                            }
                        }
                    }
                }
            }
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
        Set {
            container,
            slice,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for Set<'a, T>
where
    T: NodeIterable<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if !self.slice.is_empty() {
            let dnode = Some(T::from_raw(&self.container, self.slice[0]));
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

// ===== impl Array =====

impl<'a, S> Array<'a, S>
where
    S: Binding<'a>,
{
    pub fn new(context: &'a Context, raw: *mut S::CType) -> Array<'a, S> {
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
            let next = S::from_raw_opt(&self.context, self.raw);
            self.count -= 1;
            self.raw = unsafe { (self.raw).add(1) };
            next
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.count))
    }
}

// ===== impl SchemaModules =====

impl<'a> SchemaModules<'a> {
    pub fn new(context: &'a Context) -> SchemaModules<'a> {
        let index = 0;
        SchemaModules { context, index }
    }
}

impl<'a> Iterator for SchemaModules<'a> {
    type Item = SchemaModule<'a>;

    fn next(&mut self) -> Option<SchemaModule<'a>> {
        let rmodule = unsafe {
            ffi::ly_ctx_get_module_iter(self.context.raw, &mut self.index)
        };
        SchemaModule::from_raw_opt(&self.context, rmodule as *mut _)
    }
}

// ===== impl MetadataList =====

impl MetadataList<'_> {
    pub fn new(next: Option<Metadata>) -> MetadataList {
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
