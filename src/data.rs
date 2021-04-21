//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! YANG instance data.

use bitflags::bitflags;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::os::unix::io::AsRawFd;
use std::slice;
use std::sync::Arc;

use crate::context::Context;
use crate::error::{Error, Result};
use crate::iter::{
    Ancestors, MetadataList, NodeIterable, Set, Siblings, Traverse,
};
use crate::schema::{DataValue, SchemaModule, SchemaNode, SchemaNodeKind};
use crate::utils::*;
use libyang2_sys as ffi;

/// YANG data tree.
#[derive(Debug)]
pub struct DataTree {
    context: Arc<Context>,
    raw: *mut ffi::lyd_node,
}

/// YANG data node reference.
#[derive(Clone, Debug)]
pub struct DataNodeRef<'a> {
    tree: &'a DataTree,
    raw: *mut ffi::lyd_node,
}

/// The structure provides information about metadata of a data element. Such
/// attributes must map to annotations as specified in RFC 7952. The only
/// exception is the filter type (in NETCONF get operations) and edit-config's
/// operation attributes. In XML, they are represented as standard XML
/// attributes. In JSON, they are represented as JSON elements starting with the
/// '@' character (for more information, see the YANG metadata RFC).
#[derive(Clone, Debug)]
pub struct Metadata<'a> {
    dnode: &'a DataNodeRef<'a>,
    raw: *mut ffi::lyd_meta,
}

/// YANG data tree diff.
#[derive(Debug)]
pub struct DataDiff {
    tree: DataTree,
}

/// YANG data diff operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataDiffOp {
    Create,
    Delete,
    Replace,
}

/// Data input/output formats supported by libyang.
#[allow(clippy::upper_case_acronyms)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataFormat {
    /// XML instance data format.
    XML = ffi::LYD_FORMAT::LYD_XML,
    /// JSON instance data format.
    JSON = ffi::LYD_FORMAT::LYD_JSON,
    /// LYB instance data format.
    LYB = ffi::LYD_FORMAT::LYD_LYB,
}

bitflags! {
    /// Data parser options.
    ///
    /// Various options to change the data tree parsers behavior.
    ///
    /// Default parser behavior:
    /// - complete input file is always parsed. In case of XML, even not
    ///   well-formed XML document (multiple top-level elements) is parsed in
    ///   its entirety.
    /// - parser silently ignores data without matching schema node definition.
    /// - list instances are checked whether they have all the keys, error is
    ///   raised if not.
    ///
    /// Default parser validation behavior:
    /// - the provided data are expected to provide complete datastore content
    ///   (both the configuration and state data) and performs data validation
    ///   according to all YANG rules, specifics follow.
    /// - list instances are expected to have all the keys (it is not checked).
    /// - instantiated (status) obsolete data print a warning.
    /// - all types are fully resolved (leafref/instance-identifier targets,
    ///   unions) and must be valid (lists have all the keys, leaf(-lists)
    ///   correct values).
    /// - when statements on existing nodes are evaluated, if not satisfied, a
    ///   validation error is raised.
    /// - if-feature statements are evaluated.
    /// - invalid multiple data instances/data from several cases cause a
    ///   validation error.
    /// - implicit nodes (NP containers and default values) are added.
    pub struct DataParserFlags: u32 {
        /// Data will be only parsed and no validation will be performed. When
        /// statements are kept unevaluated, union types may not be fully
        /// resolved, if-feature statements are not checked, and default values
        /// are not added (only the ones parsed are present).
        const NO_VALIDATION = ffi::LYD_PARSE_ONLY;
        /// Instead of silently ignoring data without schema definition raise an
        /// error.
        const STRICT = ffi::LYD_PARSE_STRICT;
        /// Forbid state data in the parsed data.
        const NO_STATE = ffi::LYD_PARSE_NO_STATE;
    }
}

bitflags! {
    /// Data validation options.
    ///
    /// Various options to change data validation behaviour, both for the parser
    /// and separate validation.
    pub struct DataValidationFlags: u32 {
        /// Consider state data not allowed and raise an error if they are found.
        const NO_STATE = ffi::LYD_VALIDATE_NO_STATE;
        /// Validate only modules whose data actually exist.
        const PRESENT = ffi::LYD_VALIDATE_PRESENT;
    }
}

bitflags! {
    /// Data printer flags.
    ///
    /// Various options to change data validation behaviour, both for the parser
    /// and separate validation.
    pub struct DataPrinterFlags: u32 {
        /// Flag for printing also the (following) sibling nodes of the data
        /// node.
        const WITH_SIBLINGS = ffi::LYD_PRINT_WITHSIBLINGS;
        /// Flag for output without indentation and formatting new lines.
        const SHRINK = ffi::LYD_PRINT_SHRINK;
        /// Preserve empty non-presence containers.
        const KEEP_EMPTY_CONT = ffi::LYD_PRINT_KEEPEMPTYCONT;
        /// Explicit with-defaults mode. Only the data explicitly being present
        /// in the data tree are printed, so the implicitly added default nodes
        /// are not printed. Note that this is the default value when no WD
        /// option is specified.
        const WD_EXPLICIT = ffi::LYD_PRINT_WD_EXPLICIT;
        /// Trim mode avoids printing the nodes with the value equal to their
        /// default value.
        const WD_TRIM = ffi::LYD_PRINT_WD_TRIM;
        /// Include implicit default nodes.
        const WD_ALL = ffi::LYD_PRINT_WD_ALL;
    }
}

bitflags! {
    /// Implicit node creation options.
    ///
    /// Default behavior:
    /// - both configuration and state missing implicit nodes are added.
    /// - for existing RPC/action nodes, input implicit nodes are added.
    /// - all implicit node types are added (non-presence containers,
    ///   default leaves, and default leaf-lists).
    pub struct DataImplicitFlags: u32 {
        /// Do not add any implicit state nodes.
        const NO_STATE = ffi::LYD_IMPLICIT_NO_STATE;
        /// Do not add any implicit config nodes.
        const NO_CONFIG = ffi::LYD_IMPLICIT_NO_CONFIG;
        /// For RPC/action nodes, add output implicit nodes instead of input.
        const OUTPUT = ffi::LYD_IMPLICIT_OUTPUT;
        /// Do not add any default nodes (leaves/leaf-lists), only non-presence
        /// containers.
        const NO_DEFAULTS = ffi::LYD_IMPLICIT_NO_DEFAULTS;
    }
}

/// Methods common to data trees, data node references and data diffs.
pub trait Data {
    #[doc(hidden)]
    fn context(&self) -> &Context {
        &self.tree().context
    }

    #[doc(hidden)]
    fn tree(&self) -> &DataTree;

    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lyd_node;

    /// Search in the given data for instances of nodes matching the provided
    /// XPath.
    ///
    /// The expected format of the expression is JSON, meaning the first node in
    /// every path must have its module name as prefix or be the special `*`
    /// value for all the nodes.
    ///
    /// If a list instance is being selected with all its key values specified
    /// (but not necessarily ordered) in the form
    /// `list[key1='val1'][key2='val2'][key3='val3']` or a leaf-list instance in
    /// the form `leaf-list[.='val']`, these instances are found using hashes
    /// with constant (*O(1)*) complexity (unless they are defined in
    /// top-level). Other predicates can still follow the aforementioned ones.
    fn find_xpath(&self, xpath: &str) -> Result<Set<DataNodeRef>> {
        let xpath = CString::new(xpath).unwrap();
        let mut set = std::ptr::null_mut();
        let set_ptr = &mut set;

        let ret =
            unsafe { ffi::lyd_find_xpath(self.raw(), xpath.as_ptr(), set_ptr) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        let rnodes_count = unsafe { (*set).count } as usize;
        let slice = if rnodes_count == 0 {
            &[]
        } else {
            let rnodes = unsafe { (*set).__bindgen_anon_1.dnodes };
            unsafe { slice::from_raw_parts(rnodes, rnodes_count) }
        };

        Ok(Set::new(self.tree(), slice))
    }

    /// Search in the given data for a single node matching the provided XPath.
    ///
    /// The expected format of the expression is JSON, meaning the first node in
    /// every path must have its module name as prefix or be the special `*`
    /// value for all the nodes.
    fn find_path(&self, path: &str) -> Result<DataNodeRef> {
        let path = CString::new(path).unwrap();
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_find_path(self.raw(), path.as_ptr(), 0u8, rnode_ptr)
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(DataNodeRef::from_raw(self.tree(), rnode as *mut _))
    }

    /// Print data tree in the specified format.
    fn print_file<F: AsRawFd>(
        &self,
        fd: F,
        format: DataFormat,
        options: DataPrinterFlags,
    ) -> Result<()> {
        let ret = unsafe {
            ffi::lyd_print_fd(
                fd.as_raw_fd(),
                self.raw(),
                format as u32,
                options.bits(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(())
    }

    /// Print data tree in the specified format.
    fn print_string(
        &self,
        format: DataFormat,
        options: DataPrinterFlags,
    ) -> Result<Option<String>> {
        let mut cstr = std::ptr::null_mut();
        let cstr_ptr = &mut cstr;

        let ret = unsafe {
            ffi::lyd_print_mem(
                cstr_ptr,
                self.raw(),
                format as u32,
                options.bits(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(char_ptr_to_opt_string(cstr))
    }
}

// ===== impl DataTree =====

impl DataTree {
    /// Create new empty data tree.
    pub fn new(context: &Arc<Context>) -> DataTree {
        DataTree {
            context: context.clone(),
            raw: std::ptr::null_mut(),
        }
    }

    /// Parse (and validate) input data as a YANG data tree.
    pub fn parse_file<F: AsRawFd>(
        context: &Arc<Context>,
        fd: F,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_parse_data_fd(
                context.raw,
                fd.as_raw_fd(),
                format as u32,
                parser_options.bits(),
                validation_options.bits(),
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        Ok(DataTree::from_raw(context, rnode))
    }

    /// Parse (and validate) input data as a YANG data tree.
    pub fn parse_string(
        context: &Arc<Context>,
        data: &str,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;
        let data = CString::new(data).unwrap();

        let ret = unsafe {
            ffi::lyd_parse_data_mem(
                context.raw,
                data.as_ptr(),
                format as u32,
                parser_options.bits(),
                validation_options.bits(),
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        Ok(DataTree::from_raw(context, rnode))
    }

    /// Returns a reference to the fist top-level data node, unless the data
    /// tree is empty.
    pub fn reference(&self) -> Option<DataNodeRef> {
        if self.raw.is_null() {
            None
        } else {
            Some(DataNodeRef {
                tree: &self,
                raw: self.raw,
            })
        }
    }

    /// Create a new node or modify existing one in the data tree based on a
    /// path.
    ///
    /// If path points to a list key and the list instance does not exist,
    /// the key value from the predicate is used and value is ignored. Also,
    /// if a leaf-list is being created and both a predicate is defined in
    /// path and value is set, the predicate is preferred.
    ///
    /// For key-less lists and state leaf-lists, positional predicates can be
    /// used. If no preciate is used for these nodes, they are always created.
    ///
    /// Returns the last created or modified node (if any).
    pub fn new_path(
        &mut self,
        path: &str,
        value: Option<&str>,
    ) -> Result<Option<DataNodeRef>> {
        let path = CString::new(path).unwrap();
        let mut rnode_root = std::ptr::null_mut();
        let mut rnode = std::ptr::null_mut();
        let rnode_root_ptr = &mut rnode_root;
        let rnode_ptr = &mut rnode;
        let value_cstr;

        let (value_ptr, value_len) = match value {
            Some(value) => {
                value_cstr = CString::new(value).unwrap();
                (value_cstr.as_ptr(), value.len())
            }
            None => (std::ptr::null(), 0),
        };

        let ret = unsafe {
            ffi::lyd_new_path2(
                self.raw(),
                self.context().raw,
                path.as_ptr(),
                value_ptr as *const c_void,
                value_len as u64,
                ffi::LYD_ANYDATA_VALUETYPE::LYD_ANYDATA_STRING,
                ffi::LYD_NEW_PATH_UPDATE,
                rnode_root_ptr,
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        // Update top-level sibling.
        if self.raw.is_null() {
            self.raw = unsafe { ffi::lyd_first_sibling(rnode_root) };
        } else {
            self.raw = unsafe { ffi::lyd_first_sibling(self.raw) };
        }

        Ok(DataNodeRef::from_raw_opt(self.tree(), rnode))
    }

    /// Remove a data node.
    pub fn remove(&mut self, path: &str) -> Result<()> {
        let dnode = self.find_path(path)?;
        unsafe { ffi::lyd_free_tree(dnode.raw) };
        Ok(())
    }

    /// Fully validate the data tree.
    pub fn validate(&mut self, options: DataValidationFlags) -> Result<()> {
        let ret = unsafe {
            ffi::lyd_validate_all(
                &mut self.raw,
                self.context.raw,
                options.bits(),
                std::ptr::null_mut(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(())
    }

    /// Create a copy of the data tree.
    pub fn duplicate(&self) -> Result<DataTree> {
        let mut dup = std::ptr::null_mut();
        let dup_ptr = &mut dup;

        // Special handling for empty data trees.
        if self.raw.is_null() {
            return Ok(DataTree::from_raw(&self.context, std::ptr::null_mut()));
        }

        let options = ffi::LYD_DUP_RECURSIVE | ffi::LYD_DUP_WITH_FLAGS;
        let ret = unsafe {
            ffi::lyd_dup_siblings(
                self.raw,
                std::ptr::null_mut(),
                options,
                dup_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(DataTree::from_raw(&self.context, dup))
    }

    /// Merge the source data tree into the target data tree. Merge may not be
    /// complete until validation is called on the resulting data tree (data
    /// from more cases may be present, default and non-default values).
    pub fn merge(&mut self, source: &DataTree) -> Result<()> {
        // Special handling for empty data trees.
        if self.raw.is_null() {
            *self = source.duplicate()?;
        } else {
            let options = 0u16;
            let ret = unsafe {
                ffi::lyd_merge_siblings(&mut self.raw, source.raw, options)
            };
            if ret != ffi::LY_ERR::LY_SUCCESS {
                return Err(Error::new(&self.context));
            }
        }

        Ok(())
    }

    /// Add any missing implicit nodes. Default nodes with a false "when" are
    /// not added.
    pub fn add_implicit(&mut self, options: DataImplicitFlags) -> Result<()> {
        let ret = unsafe {
            ffi::lyd_new_implicit_all(
                &mut self.raw,
                self.context.raw,
                options.bits(),
                std::ptr::null_mut(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        // Update top-level sibling.
        self.raw = unsafe { ffi::lyd_first_sibling(self.raw) };

        Ok(())
    }

    /// Learn the differences between 2 data trees.
    ///
    /// The resulting diff is represented as a data tree with specific metadata
    /// from the internal 'yang' module. Most importantly, every node has an
    /// effective 'operation' metadata. If there is none defined on the
    /// node, it inherits the operation from the nearest parent. Top-level nodes
    /// must always have the 'operation' metadata defined. Additional
    /// metadata ('orig-default', 'value', 'orig-value', 'key', 'orig-key')
    /// are used for storing more information about the value in the first
    /// or the second tree.
    pub fn diff(&self, dtree: &DataTree) -> Result<DataDiff> {
        let options = ffi::LYD_DIFF_DEFAULTS as u16;
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_diff_siblings(self.raw, dtree.raw, options, rnode_ptr)
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(DataDiff {
            tree: DataTree::from_raw(&dtree.context, rnode),
        })
    }

    /// Apply the whole diff tree on the data tree.
    pub fn diff_apply(&mut self, diff: &DataDiff) -> Result<()> {
        let ret =
            unsafe { ffi::lyd_diff_apply_all(&mut self.raw, diff.tree.raw) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(())
    }

    /// Returns an iterator over all elements in the data tree and its sibling
    /// trees (depth-first search algorithm).
    pub fn traverse<'a>(&'a self) -> impl Iterator<Item = DataNodeRef<'a>> {
        let top = Siblings::new(self.reference());
        top.flat_map(|dnode| dnode.traverse())
    }
}

impl Data for DataTree {
    fn tree(&self) -> &DataTree {
        &self
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.raw
    }
}

impl<'a> Binding<'a> for DataTree {
    type CType = ffi::lyd_node;
    type Container = Arc<Context>;

    fn from_raw(
        context: &'a Arc<Context>,
        raw: *mut ffi::lyd_node,
    ) -> DataTree {
        DataTree {
            context: context.clone(),
            raw,
        }
    }
}

unsafe impl Send for DataTree {}
unsafe impl Sync for DataTree {}

impl Drop for DataTree {
    fn drop(&mut self) {
        unsafe { ffi::lyd_free_all(self.raw) };
    }
}

// ===== impl DataNodeRef =====

impl<'a> DataNodeRef<'a> {
    /// Schema definition of this node.
    pub fn schema(&self) -> SchemaNode {
        let raw = unsafe { (*self.raw).schema };
        SchemaNode::from_raw(self.context(), raw as *mut _)
    }

    /// Get the owner module of the data node. It is the module of the top-level
    /// schema node. Generally, in case of augments it is the target module,
    /// recursively, otherwise it is the module where the data node is defined.
    pub fn owner_module(&self) -> SchemaModule {
        let module = unsafe { ffi::lyd_owner_module(self.raw()) };
        SchemaModule::from_raw(self.context(), module as *mut _)
    }

    /// Returns an iterator over the ancestor data nodes.
    pub fn ancestors(&self) -> Ancestors<'a, DataNodeRef<'a>> {
        let parent = self.parent();
        Ancestors::new(parent)
    }

    /// Returns an iterator over this data node and its ancestors.
    pub fn inclusive_ancestors(&self) -> Ancestors<'a, DataNodeRef<'a>> {
        Ancestors::new(Some(self.clone()))
    }

    /// Returns an iterator over the sibling data nodes.
    pub fn siblings(&self) -> Siblings<'a, DataNodeRef<'a>> {
        let sibling = self.next_sibling();
        Siblings::new(sibling)
    }

    /// Returns an iterator over this data node and its siblings.
    pub fn inclusive_siblings(&self) -> Siblings<'a, DataNodeRef<'a>> {
        Siblings::new(Some(self.clone()))
    }

    /// Returns an iterator over the child data nodes.
    pub fn children(&self) -> Siblings<'a, DataNodeRef<'a>> {
        let child = self.first_child();
        Siblings::new(child)
    }

    /// Returns an iterator over all elements in the data tree (depth-first
    /// search algorithm).
    pub fn traverse(&self) -> Traverse<'a, DataNodeRef<'a>> {
        Traverse::new(self.clone())
    }

    /// Returns an iterator over the keys of the list.
    pub fn list_keys(&self) -> impl Iterator<Item = DataNodeRef<'a>> {
        self.children().filter(|dnode| dnode.schema().is_list_key())
    }

    /// Returns an iterator over all metadata associated to this node.
    pub fn meta(&self) -> MetadataList {
        let rmeta = unsafe { (*self.raw).meta };
        let meta = Metadata::from_raw_opt(&self, rmeta);
        MetadataList::new(meta)
    }

    /// Generate path of the given node.
    pub fn path(&self) -> String {
        let mut buf: [c_char; 4096] = [0; 4096];

        let pathtype = ffi::LYD_PATH_TYPE::LYD_PATH_STD;
        let ret = unsafe {
            ffi::lyd_path(
                self.raw,
                pathtype,
                buf.as_mut_ptr(),
                buf.len() as u64,
            )
        };
        if ret.is_null() {
            panic!("Failed to generate path of the data node");
        }

        char_ptr_to_string(buf.as_ptr())
    }

    /// Node's value (canonical string representation).
    pub fn value_canonical(&self) -> Option<String> {
        match self.schema().kind() {
            SchemaNodeKind::Leaf | SchemaNodeKind::LeafList => {
                let rnode = self.raw as *mut ffi::lyd_node_term;
                let value = unsafe { (*rnode).value._canonical };
                char_ptr_to_opt_string(value)
            }
            _ => None,
        }
    }

    /// Node's value (typed representation).
    pub fn value(&self) -> Option<DataValue> {
        match self.schema().kind() {
            SchemaNodeKind::Leaf | SchemaNodeKind::LeafList => {
                let rnode = self.raw as *const ffi::lyd_node_term;
                let rvalue = unsafe { (*rnode).value };
                let value = unsafe { DataValue::from_raw(&rvalue) };
                Some(value)
            }
            _ => None,
        }
    }

    /// Check whether a node value equals to its default one.
    pub fn is_default(&self) -> bool {
        match self.schema().kind() {
            SchemaNodeKind::Leaf | SchemaNodeKind::LeafList => {
                (unsafe { ffi::lyd_is_default(self.raw) }) != 0
            }
            _ => false,
        }
    }

    /// Set private user data, not used by libyang.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided pointer is valid.
    pub unsafe fn set_private(&mut self, ptr: *mut c_void) {
        (*self.raw).priv_ = ptr;
    }

    /// Get private user data, not used by libyang.
    pub fn get_private(&self) -> Option<*mut c_void> {
        let priv_ = unsafe { (*self.raw).priv_ };
        if priv_.is_null() {
            None
        } else {
            Some(priv_)
        }
    }
}

impl<'a> Data for DataNodeRef<'a> {
    fn tree(&self) -> &DataTree {
        &self.tree
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.raw
    }
}

impl<'a> Binding<'a> for DataNodeRef<'a> {
    type CType = ffi::lyd_node;
    type Container = DataTree;

    fn from_raw(
        tree: &'a DataTree,
        raw: *mut ffi::lyd_node,
    ) -> DataNodeRef<'a> {
        DataNodeRef { tree, raw }
    }
}

impl<'a> NodeIterable<'a> for DataNodeRef<'a> {
    fn parent(&self) -> Option<DataNodeRef<'a>> {
        // NOTE: can't use lyd_parent() since it's an inline function.
        let rparent =
            unsafe { &mut (*(*self.raw).parent).__bindgen_anon_1.node };
        DataNodeRef::from_raw_opt(&self.tree, rparent)
    }

    fn next_sibling(&self) -> Option<DataNodeRef<'a>> {
        let rsibling = unsafe { (*self.raw).next };
        DataNodeRef::from_raw_opt(&self.tree, rsibling)
    }

    fn first_child(&self) -> Option<DataNodeRef<'a>> {
        // NOTE: can't use lyd_child() since it's an inline function.
        let snode = unsafe { (*self.raw).schema };
        if snode.is_null() {
            let ropaq = self.raw as *mut ffi::lyd_node_opaq;
            let rchild = unsafe { (*ropaq).child };
            return DataNodeRef::from_raw_opt(&self.tree, rchild);
        }

        let nodetype = unsafe { (*snode).nodetype as u32 };
        let rchild = match nodetype {
            ffi::LYS_CONTAINER
            | ffi::LYS_LIST
            | ffi::LYS_RPC
            | ffi::LYS_ACTION
            | ffi::LYS_NOTIF => {
                let rinner = self.raw as *mut ffi::lyd_node_inner;
                unsafe { (*rinner).child }
            }
            _ => std::ptr::null_mut(),
        };
        DataNodeRef::from_raw_opt(&self.tree, rchild)
    }
}

impl<'a> PartialEq for DataNodeRef<'a> {
    fn eq(&self, other: &DataNodeRef) -> bool {
        self.raw == other.raw
    }
}

unsafe impl Send for DataNodeRef<'_> {}
unsafe impl Sync for DataNodeRef<'_> {}

// ===== impl Metadata =====

impl<'a> Metadata<'a> {
    /// Metadata name.
    pub fn name(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).name })
    }

    /// Metadata value representation.
    pub fn value(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).value._canonical })
    }

    /// Next metadata.
    #[doc(hidden)]
    pub(crate) fn next(&self) -> Option<Metadata<'a>> {
        let rnext = unsafe { (*self.raw).next };
        Metadata::from_raw_opt(&self.dnode, rnext)
    }
}

impl<'a> Binding<'a> for Metadata<'a> {
    type CType = ffi::lyd_meta;
    type Container = DataNodeRef<'a>;

    fn from_raw(
        dnode: &'a DataNodeRef,
        raw: *mut ffi::lyd_meta,
    ) -> Metadata<'a> {
        Metadata { dnode, raw }
    }
}

impl<'a> PartialEq for Metadata<'a> {
    fn eq(&self, other: &Metadata) -> bool {
        self.raw == other.raw
    }
}

unsafe impl Send for Metadata<'_> {}
unsafe impl Sync for Metadata<'_> {}

// ===== impl DataDiff =====

impl DataDiff {
    /// Returns an iterator over the data changes.
    pub fn iter(&self) -> impl Iterator<Item = (DataDiffOp, DataNodeRef<'_>)> {
        self.tree.traverse().filter_map(|dnode| {
            match dnode.meta().find(|meta| meta.name() == "operation") {
                Some(meta) => match meta.value() {
                    "create" => Some((DataDiffOp::Create, dnode)),
                    "delete" => Some((DataDiffOp::Delete, dnode)),
                    "replace" => Some((DataDiffOp::Replace, dnode)),
                    "none" => None,
                    _ => unreachable!(),
                },
                None => None,
            }
        })
    }

    /// Reverse a diff and make the opposite changes. Meaning change create to
    /// delete, delete to create, or move from place A to B to move from B
    /// to A and so on.
    pub fn reverse(&self) -> Result<DataDiff> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret =
            unsafe { ffi::lyd_diff_reverse_all(self.tree.raw, rnode_ptr) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.tree.context));
        }

        Ok(DataDiff {
            tree: DataTree::from_raw(&self.tree.context, rnode),
        })
    }
}

impl Data for DataDiff {
    fn tree(&self) -> &DataTree {
        &self.tree
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.tree.raw
    }
}
