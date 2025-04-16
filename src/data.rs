//
// Copyright (c) The yang-rs Core Contributors
//
// SPDX-License-Identifier: MIT
//

//! YANG instance data.

use bitflags::bitflags;
use core::ffi::{c_char, c_void};
use std::ffi::CStr;
use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::slice;

use crate::context::Context;
use crate::error::{Error, Result};
use crate::iter::{
    Ancestors, MetadataList, NodeIterable, Set, Siblings, Traverse,
};
use crate::schema::SchemaExtInstance;
use crate::schema::{DataValue, SchemaModule, SchemaNode, SchemaNodeKind};
use crate::utils::*;
use libyang3_sys as ffi;

/// YANG data tree.
#[derive(Debug)]
pub struct DataTree<'a> {
    context: &'a Context,
    raw: *mut ffi::lyd_node,
}

/// YANG data tree with an associated inner node reference.
#[derive(Debug)]
pub struct DataTreeOwningRef<'a> {
    pub tree: DataTree<'a>,
    raw: *mut ffi::lyd_node,
}

/// YANG data node reference.
#[derive(Clone, Debug)]
pub struct DataNodeRef<'a> {
    tree: &'a DataTree<'a>,
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
pub struct DataDiff<'a> {
    tree: DataTree<'a>,
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

/// Data operation type.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataOperation {
    /// Generic YANG instance data.
    Data = ffi::lyd_type::LYD_TYPE_DATA_YANG,
    /// Instance of a YANG RPC/action request with only "input" data children.
    /// Including all parents in case of an action
    RpcYang = ffi::lyd_type::LYD_TYPE_RPC_YANG,
    /// Instance of a YANG notification, including all parents in case of a
    /// nested one.
    NotificationYang = ffi::lyd_type::LYD_TYPE_NOTIF_YANG,
    /// Instance of a YANG RPC/action request with only "output" data children.
    /// Including all parents in case of an action
    ReplyYang = ffi::lyd_type::LYD_TYPE_REPLY_YANG,
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

bitflags! {
    /// Data diff options.
    ///
    /// Default behavior:
    /// - Any default nodes are treated as non-existent and ignored.
    pub struct DataDiffFlags: u16 {
        /// Default nodes in the trees are not ignored but treated similarly to
        /// explicit nodes. Also, leaves and leaf-lists are added into diff even
        /// in case only their default flag (state) was changed.
        const DEFAULTS = ffi::LYD_DIFF_DEFAULTS as u16;
    }
}

/// Methods common to data trees, data node references and data diffs.
pub trait Data<'a> {
    #[doc(hidden)]
    fn context(&self) -> &'a Context {
        self.tree().context
    }

    #[doc(hidden)]
    fn tree(&self) -> &DataTree<'a>;

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
    fn find_xpath(&'a self, xpath: &str) -> Result<Set<'a, DataNodeRef<'a>>> {
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
    fn find_path(&'a self, path: &str) -> Result<DataNodeRef<'a>> {
        let path = CString::new(path).unwrap();
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_find_path(self.raw(), path.as_ptr(), 0u8, rnode_ptr)
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(unsafe { DataNodeRef::from_raw(self.tree(), rnode as *mut _) })
    }

    /// Search in the given data for a single node matching the provided XPath and
    /// is an RPC/ output nodes or in input nodes.
    ///
    /// The expected format of the expression is JSON, meaning the first node in
    /// every path must have its module name as prefix or be the special `*`
    /// value for all the nodes.
    fn find_output_path(&'a self, path: &str) -> Result<DataNodeRef<'a>> {
        let path = CString::new(path).unwrap();
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_find_path(self.raw(), path.as_ptr(), 1u8, rnode_ptr)
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(unsafe { DataNodeRef::from_raw(self.tree(), rnode as *mut _) })
    }

    /// Print data tree in the specified format.
    #[cfg(not(target_os = "windows"))]
    fn print_file<F: std::os::unix::io::AsRawFd>(
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
    #[cfg(target_os = "windows")]
    fn print_file(
        &self,
        file: impl std::os::windows::io::AsRawHandle,
        format: DataFormat,
        options: DataPrinterFlags,
    ) -> Result<()> {
        use libc::open_osfhandle;

        let raw_handle = file.as_raw_handle();

        let fd = unsafe { open_osfhandle(raw_handle as isize, 0) };

        let ret = unsafe {
            ffi::lyd_print_fd(fd, self.raw(), format as u32, options.bits())
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(())
    }

    /// Print data tree in the specified format to a `String`.
    ///
    /// # Warning
    /// For printing a data tree in the `DataFormat::LYB` format, use the
    /// [`Data::print_bytes`] method instead. Using this function with
    /// `DataFormat::LYB` may result in mangled data because the `LYB` format
    /// can contain invalid UTF-8 sequences, which cannot be represented in a
    /// `String`.
    fn print_string(
        &self,
        format: DataFormat,
        options: DataPrinterFlags,
    ) -> Result<String> {
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

        Ok(char_ptr_to_string(cstr, true))
    }

    /// Print data tree in the specified format to a bytes vector.
    fn print_bytes(
        &self,
        format: DataFormat,
        options: DataPrinterFlags,
    ) -> Result<Vec<u8>> {
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

        let bytes = match format {
            DataFormat::XML | DataFormat::JSON => {
                // Convert the null-terminated C string to a vector of bytes.
                // After converting to bytes, manually add a null terminator.
                let mut bytes =
                    unsafe { CStr::from_ptr(cstr) }.to_bytes().to_vec();
                bytes.push(0);
                bytes
            }
            DataFormat::LYB => {
                // Get the length of the LYB data.
                let len = unsafe { ffi::lyd_lyb_data_length(cstr) };
                // For the LYB data format, `cstr` isn't null-terminated.
                // Create a byte slice from the raw parts and convert it to a
                // vector.
                unsafe { std::slice::from_raw_parts(cstr as _, len as _) }
                    .to_vec()
            }
        };
        Ok(bytes)
    }
}

// ===== impl DataTree =====

enum CtxOrExt<'a> {
    C(&'a Context),
    E(&'a SchemaExtInstance<'a>),
}

impl<'a> DataTree<'a> {
    /// Create new empty data tree.
    pub fn new(context: &'a Context) -> DataTree<'a> {
        DataTree {
            context,
            raw: std::ptr::null_mut(),
        }
    }

    /// Returns a mutable raw pointer to the underlying C library representation
    /// of the root node of the YANG data tree.
    pub fn into_raw(self) -> *mut ffi::lyd_node {
        ManuallyDrop::new(self).raw
    }

    unsafe fn reroot(&mut self, raw: *mut ffi::lyd_node) {
        if self.raw.is_null() {
            let mut dnode = DataNodeRef::from_raw(self, raw);
            while let Some(parent) = dnode.parent() {
                dnode = parent;
            }
            self.raw = dnode.raw();
        }
        self.raw = ffi::lyd_first_sibling(self.raw);
    }

    /// Parse (and validate) input data as a YANG data tree.
    #[cfg(not(target_os = "windows"))]
    pub fn parse_file<F: std::os::unix::io::AsRawFd>(
        context: &'a Context,
        fd: F,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree<'a>> {
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

        Ok(unsafe { DataTree::from_raw(context, rnode) })
    }
    #[cfg(target_os = "windows")]
    pub fn parse_file(
        context: &'a Context,
        file: impl std::os::windows::io::AsRawHandle,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree<'a>> {
        use libc::open_osfhandle;

        let raw_handle = file.as_raw_handle();

        let fd = unsafe { open_osfhandle(raw_handle as isize, 0) };

        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_parse_data_fd(
                context.raw,
                fd,
                format as u32,
                parser_options.bits(),
                validation_options.bits(),
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        Ok(unsafe { DataTree::from_raw(context, rnode) })
    }

    fn _parse_string(
        ctx_or_ext: CtxOrExt<'a>,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree<'a>> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;
        let context = match ctx_or_ext {
            CtxOrExt::C(c) => c,
            CtxOrExt::E(e) => e.context,
        };

        // Create input handler.
        let cdata;
        let mut ly_in = std::ptr::null_mut();
        let ret = match format {
            DataFormat::XML | DataFormat::JSON => unsafe {
                cdata = CString::new(data.as_ref()).unwrap();
                ffi::ly_in_new_memory(cdata.as_ptr() as _, &mut ly_in)
            },
            DataFormat::LYB => unsafe {
                ffi::ly_in_new_memory(data.as_ref().as_ptr() as _, &mut ly_in)
            },
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        let ret = unsafe {
            match ctx_or_ext {
                CtxOrExt::C(c) => ffi::lyd_parse_data(
                    c.raw,
                    std::ptr::null_mut(),
                    ly_in,
                    format as u32,
                    parser_options.bits(),
                    validation_options.bits(),
                    rnode_ptr,
                ),
                CtxOrExt::E(e) => ffi::lyd_parse_ext_data(
                    e.raw,
                    std::ptr::null_mut(),
                    ly_in,
                    format as u32,
                    parser_options.bits(),
                    validation_options.bits(),
                    rnode_ptr,
                ),
            }
        };
        unsafe { ffi::ly_in_free(ly_in, 0) };

        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        Ok(unsafe { DataTree::from_raw(context, rnode) })
    }

    /// Parse (and validate) input data as a YANG data tree.
    pub fn parse_string(
        context: &'a Context,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree<'a>> {
        DataTree::_parse_string(
            CtxOrExt::C(context),
            data,
            format,
            parser_options,
            validation_options,
        )
    }

    /// Parse input data as an extension data tree using the given schema
    /// extension.
    pub fn parse_ext_string(
        ext: &'a SchemaExtInstance<'a>,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataTree<'a>> {
        DataTree::_parse_string(
            CtxOrExt::E(ext),
            data,
            format,
            parser_options,
            validation_options,
        )
    }

    fn _parse_op_string(
        ctx_or_ext: CtxOrExt<'a>,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        op: DataOperation,
    ) -> Result<DataTree<'a>> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;
        let context = match ctx_or_ext {
            CtxOrExt::C(c) => c,
            CtxOrExt::E(e) => e.context,
        };

        // Create input handler.
        let cdata = CString::new(data.as_ref()).unwrap();
        let mut ly_in = std::ptr::null_mut();
        let ret =
            unsafe { ffi::ly_in_new_memory(cdata.as_ptr() as _, &mut ly_in) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        let ret = unsafe {
            match ctx_or_ext {
                CtxOrExt::C(c) => ffi::lyd_parse_op(
                    c.raw,
                    std::ptr::null_mut(),
                    ly_in,
                    format as u32,
                    op as u32,
                    rnode_ptr,
                    std::ptr::null_mut(),
                ),
                CtxOrExt::E(e) => ffi::lyd_parse_ext_op(
                    e.raw,
                    std::ptr::null_mut(),
                    ly_in,
                    format as u32,
                    op as u32,
                    rnode_ptr,
                    std::ptr::null_mut(),
                ),
            }
        };
        unsafe { ffi::ly_in_free(ly_in, 0) };

        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        Ok(unsafe { DataTree::from_raw(context, rnode) })
    }

    /// Parse YANG data into an operation data tree.
    pub fn parse_op_string(
        context: &'a Context,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        op: DataOperation,
    ) -> Result<DataTree<'a>> {
        DataTree::_parse_op_string(CtxOrExt::C(context), data, format, op)
    }

    /// Parse op data as an extension data tree using the given schema
    /// extension. Parse input data into an operation data tree.
    pub fn parse_op_ext_string(
        ext: &'a SchemaExtInstance<'a>,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        op: DataOperation,
    ) -> Result<DataTree<'a>> {
        DataTree::_parse_op_string(CtxOrExt::E(ext), data, format, op)
    }

    /// Returns a reference to the fist top-level data node, unless the data
    /// tree is empty.
    pub fn reference(&self) -> Option<DataNodeRef<'_>> {
        if self.raw.is_null() {
            None
        } else {
            Some(DataNodeRef {
                tree: self,
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
    /// The output parameter can be used to change the behavior to ignore
    /// RPC/action input schema nodes and use only output ones.
    ///
    /// Returns the last created or modified node (if any).
    pub fn new_path(
        &mut self,
        path: &str,
        value: Option<&str>,
        output: bool,
    ) -> Result<Option<DataNodeRef<'_>>> {
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

        let mut options = ffi::LYD_NEW_PATH_UPDATE;
        if output {
            options |= ffi::LYD_NEW_VAL_OUTPUT;
        }

        let ret = unsafe {
            ffi::lyd_new_path2(
                self.raw(),
                self.context().raw,
                path.as_ptr(),
                value_ptr as *const c_void,
                value_len,
                ffi::LYD_ANYDATA_VALUETYPE::LYD_ANYDATA_STRING,
                options,
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

        Ok(unsafe { DataNodeRef::from_raw_opt(self.tree(), rnode) })
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
            return Err(Error::new(self.context));
        }

        Ok(())
    }

    /// Create a copy of the data tree.
    pub fn duplicate<'b>(&'b self) -> Result<DataTree<'a>> {
        let mut dup = std::ptr::null_mut();
        let dup_ptr = &mut dup;

        // Special handling for empty data trees.
        if self.raw.is_null() {
            return Ok(unsafe {
                DataTree::from_raw(self.context, std::ptr::null_mut())
            });
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
            return Err(Error::new(self.context));
        }

        Ok(unsafe { DataTree::from_raw(self.context, dup) })
    }

    /// Merge the source data tree into the target data tree. Merge may not be
    /// complete until validation is called on the resulting data tree (data
    /// from more cases may be present, default and non-default values).
    pub fn merge(&mut self, source: &DataTree<'_>) -> Result<()> {
        // Special handling for empty data trees.
        if self.raw.is_null() {
            let mut new_tree = source.duplicate()?;
            self.raw = new_tree.raw;
            new_tree.raw = std::ptr::null_mut();
        } else {
            let options = 0u16;
            let ret = unsafe {
                ffi::lyd_merge_siblings(&mut self.raw, source.raw, options)
            };
            if ret != ffi::LY_ERR::LY_SUCCESS {
                return Err(Error::new(self.context));
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
            return Err(Error::new(self.context));
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
    pub fn diff(
        &self,
        dtree: &DataTree<'a>,
        options: DataDiffFlags,
    ) -> Result<DataDiff<'a>> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_diff_siblings(
                self.raw,
                dtree.raw,
                options.bits(),
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context));
        }

        Ok(DataDiff {
            tree: unsafe { DataTree::from_raw(dtree.context, rnode) },
        })
    }

    /// Apply the whole diff tree on the data tree.
    pub fn diff_apply(&mut self, diff: &DataDiff<'a>) -> Result<()> {
        let ret =
            unsafe { ffi::lyd_diff_apply_all(&mut self.raw, diff.tree.raw) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context));
        }

        Ok(())
    }

    /// Returns an iterator over all elements in the data tree and its sibling
    /// trees (depth-first search algorithm).
    pub fn traverse(&self) -> impl Iterator<Item = DataNodeRef<'_>> {
        let top = Siblings::new(self.reference());
        top.flat_map(|dnode| dnode.traverse())
    }
}

impl<'a> Data<'a> for DataTree<'a> {
    fn tree(&self) -> &DataTree<'a> {
        self
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.raw
    }
}

unsafe impl<'a> Binding<'a> for DataTree<'a> {
    type CType = ffi::lyd_node;
    type Container = Context;

    unsafe fn from_raw(
        context: &'a Context,
        raw: *mut ffi::lyd_node,
    ) -> DataTree<'a> {
        DataTree { context, raw }
    }
}

unsafe impl Send for DataTree<'_> {}
unsafe impl Sync for DataTree<'_> {}

impl Drop for DataTree<'_> {
    fn drop(&mut self) {
        unsafe { ffi::lyd_free_all(self.raw) };
    }
}

// ===== impl DataTreeOwningRef =====

impl<'a> DataTreeOwningRef<'a> {
    unsafe fn from_raw(tree: DataTree<'a>, raw: *mut ffi::lyd_node) -> Self {
        DataTreeOwningRef { tree, raw }
    }

    /// Get a temporary DataTreeOwningRef from a raw lyd_node pointer.
    ///
    /// The intent of this function is to create a temporary DataTreeOwningRef
    /// from a raw lyd_node pointer passed to user that lives in a DataTree they
    /// do not own (e.g., as a C callback argument).
    ///
    /// # Safety:
    ///
    /// The user must still have access and pass in the context that the tree
    /// of the data node was created from. The function will panic if the
    /// context does not match.
    ///
    /// The returned value is only valid for as long as the node's tree and the
    /// node itself are valid.
    pub unsafe fn from_raw_node(
        context: &Context,
        raw: *mut ffi::lyd_node,
    ) -> std::mem::ManuallyDrop<DataTreeOwningRef<'_>> {
        if (*(*(*raw).schema).module).ctx != context.raw {
            panic!("raw node context differs from passed in context");
        }
        let mut tree = DataTree::new(context);
        tree.reroot(raw);
        std::mem::ManuallyDrop::new(DataTreeOwningRef { tree, raw })
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
    /// The output parameter can be used to change the behavior to ignore
    /// RPC/action input schema nodes and use only output ones.
    ///
    /// Returns the last created or modified node (if any).
    pub fn new_path(
        context: &'a Context,
        path: &str,
        value: Option<&str>,
        output: bool,
    ) -> Result<Self> {
        let mut tree = DataTree::new(context);
        let raw = {
            match tree.new_path(path, value, output)? {
                Some(node) => node.raw,
                None => tree.find_path(path)?.raw,
            }
        };
        Ok(unsafe { DataTreeOwningRef::from_raw(tree, raw) })
    }

    /// Obtain a DataNodeRef that the DataTreeOwningRef is referencing.
    pub fn noderef(&'a self) -> DataNodeRef<'a> {
        DataNodeRef {
            tree: &self.tree,
            raw: self.raw,
        }
    }

    fn _parse_op(
        context: &'a Context,
        raw: *mut ffi::lyd_node,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        op_type: ffi::lyd_type::Type,
        op_node_ptr: *mut *mut ffi::lyd_node,
    ) -> Result<()> {
        let mut opaque = std::ptr::null_mut();
        let opaque_ptr = &mut opaque;

        // Create input handler.
        if format != DataFormat::JSON && format != DataFormat::XML {
            return Err(Error {
                errcode: ffi::LY_ERR::LY_EINVAL,
                ..Default::default()
            });
        }
        let mut ly_in = std::ptr::null_mut();
        let cdata = CString::new(data.as_ref()).unwrap();
        let ret =
            unsafe { ffi::ly_in_new_memory(cdata.as_ptr() as _, &mut ly_in) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        let ret = unsafe {
            ffi::lyd_parse_op(
                context.raw,
                raw,
                ly_in,
                format as u32,
                op_type,
                opaque_ptr,
                op_node_ptr,
            )
        };

        unsafe { ffi::ly_in_free(ly_in, 0) };
        unsafe { ffi::lyd_free_all(opaque) }; // Can be set on error.

        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(context));
        }

        Ok(())
    }

    /// Parse RPC with input args from NETCONF (i.e. in XML)
    pub fn parse_netconf_rpc_op(
        context: &'a Context,
        data: impl AsRef<[u8]>,
    ) -> Result<DataTreeOwningRef<'a>> {
        let mut tree = DataTreeOwningRef {
            tree: DataTree::new(context),
            raw: std::ptr::null_mut(),
        };

        Self::_parse_op(
            context,
            std::ptr::null_mut(),
            data,
            DataFormat::XML,
            ffi::lyd_type::LYD_TYPE_RPC_NETCONF,
            &mut tree.raw,
        )?;
        unsafe { tree.tree.reroot(tree.raw) };
        Ok(tree)
    }

    /// Parse RPC REPLY with output args from NETCONF (in XML)
    pub fn parse_netconf_reply_op(
        &mut self,
        data: impl AsRef<[u8]>,
    ) -> Result<()> {
        Self::_parse_op(
            self.tree.context,
            self.raw,
            data,
            DataFormat::XML,
            ffi::lyd_type::LYD_TYPE_REPLY_NETCONF,
            std::ptr::null_mut(),
        )
    }

    /// Parse NOTIFICATION with args from NETCONF (i.e. in XML)
    pub fn parse_netconf_notif_op(
        context: &'a Context,
        data: impl AsRef<[u8]>,
    ) -> Result<DataTreeOwningRef<'a>> {
        let mut tree = DataTreeOwningRef {
            tree: DataTree::new(context),
            raw: std::ptr::null_mut(),
        };

        Self::_parse_op(
            context,
            std::ptr::null_mut(),
            data,
            DataFormat::XML,
            ffi::lyd_type::LYD_TYPE_NOTIF_NETCONF,
            &mut tree.raw,
        )?;
        unsafe { tree.tree.reroot(tree.raw) };
        Ok(tree)
    }

    /// Parse RPC with input args from RESTCONF (in JSON or XML)
    pub fn parse_restconf_rpc_op(
        &mut self,
        data: impl AsRef<[u8]>,
        format: DataFormat,
    ) -> Result<()> {
        Self::_parse_op(
            self.tree.context,
            self.raw,
            data,
            format,
            ffi::lyd_type::LYD_TYPE_RPC_RESTCONF,
            std::ptr::null_mut(),
        )
    }

    /// Parse RPC REPLY with output args from RESTCONF (in JSON or XML)
    pub fn parse_restconf_reply_op(
        &mut self,
        data: impl AsRef<[u8]>,
        format: DataFormat,
    ) -> Result<()> {
        Self::_parse_op(
            self.tree.context,
            self.raw,
            data,
            format,
            ffi::lyd_type::LYD_TYPE_REPLY_RESTCONF,
            std::ptr::null_mut(),
        )
    }

    /// Parse NOTIFICATION with args from RESTCONF (in either JSON or XML)
    pub fn parse_restconf_notif_op(
        context: &'a Context,
        data: impl AsRef<[u8]>,
        format: DataFormat,
    ) -> Result<DataTreeOwningRef<'a>> {
        let mut tree = DataTreeOwningRef {
            tree: DataTree::new(context),
            raw: std::ptr::null_mut(),
        };

        if format == DataFormat::XML {
            Self::_parse_op(
                context,
                std::ptr::null_mut(),
                data,
                DataFormat::XML,
                ffi::lyd_type::LYD_TYPE_NOTIF_NETCONF,
                &mut tree.raw,
            )?;
        } else {
            Self::_parse_op(
                context,
                std::ptr::null_mut(),
                data,
                DataFormat::JSON,
                ffi::lyd_type::LYD_TYPE_NOTIF_RESTCONF,
                &mut tree.raw,
            )?;
        }
        unsafe { tree.tree.reroot(tree.raw) };
        Ok(tree)
    }
}

impl<'a> From<DataTree<'a>> for DataTreeOwningRef<'a> {
    fn from(tree: DataTree<'a>) -> DataTreeOwningRef<'a> {
        let raw = tree.raw;
        unsafe { DataTreeOwningRef::from_raw(tree, raw) }
    }
}

impl<'a> From<&'a DataTreeOwningRef<'_>> for DataNodeRef<'a> {
    fn from(tree: &'a DataTreeOwningRef<'_>) -> DataNodeRef<'a> {
        DataNodeRef {
            tree: &tree.tree,
            raw: tree.raw,
        }
    }
}

impl<'a> Data<'a> for DataTreeOwningRef<'a> {
    fn tree(&self) -> &DataTree<'a> {
        &self.tree
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.raw
    }
}

unsafe impl Send for DataTreeOwningRef<'_> {}
unsafe impl Sync for DataTreeOwningRef<'_> {}

// ===== impl DataNodeRef =====

impl<'a> DataNodeRef<'a> {
    /// Returns a mutable raw pointer to the underlying C library representation
    /// of the data node reference.
    pub fn as_raw(&self) -> *mut ffi::lyd_node {
        self.raw
    }

    /// Schema definition of this node.
    pub fn schema(&self) -> SchemaNode<'_> {
        let raw = unsafe { (*self.raw).schema };
        unsafe { SchemaNode::from_raw(self.context(), raw as *mut _) }
    }

    /// Get the owner module of the data node. It is the module of the top-level
    /// schema node. Generally, in case of augments it is the target module,
    /// recursively, otherwise it is the module where the data node is defined.
    pub fn owner_module(&self) -> SchemaModule<'_> {
        let module = unsafe { ffi::lyd_owner_module(self.raw()) };
        unsafe { SchemaModule::from_raw(self.context(), module as *mut _) }
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
    pub fn meta(&self) -> MetadataList<'_> {
        let rmeta = unsafe { (*self.raw).meta };
        let meta = unsafe { Metadata::from_raw_opt(self, rmeta) };
        MetadataList::new(meta)
    }

    /// Generate path of the given node.
    pub fn path(&self) -> String {
        let mut buf: [c_char; 4096] = [0; 4096];

        let pathtype = ffi::LYD_PATH_TYPE::LYD_PATH_STD;
        let ret = unsafe {
            ffi::lyd_path(self.raw, pathtype, buf.as_mut_ptr(), buf.len())
        };
        if ret.is_null() {
            panic!("Failed to generate path of the data node");
        }

        char_ptr_to_string(buf.as_ptr(), false)
    }

    /// Node's value (canonical string representation).
    pub fn value_canonical(&self) -> Option<String> {
        match self.schema().kind() {
            SchemaNodeKind::Leaf | SchemaNodeKind::LeafList => {
                let rnode = self.raw as *mut ffi::lyd_node_term;
                let mut value = unsafe { (*rnode).value._canonical };
                if value.is_null() {
                    value = unsafe {
                        ffi::lyd_value_get_canonical(
                            self.context().raw,
                            &(*rnode).value,
                        )
                    };
                }
                char_ptr_to_opt_string(value, false)
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
                let value =
                    unsafe { DataValue::from_raw(self.tree.context, &rvalue) };
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

    /// Create a copy of the data subtree.
    ///
    /// When the `with_parents` parameter is set, duplicate also all the node
    /// parents. Keys are also duplicated for lists.
    pub fn duplicate(&self, with_parents: bool) -> Result<DataTree<'a>> {
        let mut dup = std::ptr::null_mut();
        let dup_ptr = &mut dup;

        // Special handling for empty data trees.
        if self.raw.is_null() {
            return Ok(unsafe {
                DataTree::from_raw(self.tree.context, std::ptr::null_mut())
            });
        }

        let mut options = ffi::LYD_DUP_RECURSIVE | ffi::LYD_DUP_WITH_FLAGS;
        if with_parents {
            options |= ffi::LYD_DUP_WITH_PARENTS;
        }
        let ret = unsafe {
            ffi::lyd_dup_single(
                self.raw,
                std::ptr::null_mut(),
                options,
                dup_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        if with_parents {
            let mut dnode = unsafe { DataNodeRef::from_raw(self.tree, dup) };
            while let Some(parent) = dnode.parent() {
                dnode = parent;
            }
            dup = dnode.raw();
        }

        Ok(unsafe { DataTree::from_raw(self.tree.context, dup) })
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

    /// Create a new inner node (container, notification, RPC or action) in the
    /// data tree.
    ///
    /// Returns the created node.
    pub fn new_inner(
        &mut self,
        module: Option<&SchemaModule<'_>>,
        name: &str,
    ) -> Result<DataNodeRef<'a>> {
        let name_cstr = CString::new(name).unwrap();
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret = unsafe {
            ffi::lyd_new_inner(
                self.raw(),
                module
                    .map(|module| module.as_raw())
                    .unwrap_or(std::ptr::null_mut()),
                name_cstr.as_ptr(),
                0,
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(unsafe { DataNodeRef::from_raw(self.tree, rnode) })
    }

    /// Create a new list node in the data tree.
    ///
    /// The `keys` parameter should be a string containing key-value pairs in
    /// the format:"[key1='val1'][key2='val2']...". The order of the key-value
    /// pairs does not matter.
    ///
    /// Returns the created node.
    pub fn new_list(
        &mut self,
        module: Option<&SchemaModule<'_>>,
        name: &str,
        keys: &str,
    ) -> Result<DataNodeRef<'a>> {
        let name_cstr = CString::new(name).unwrap();
        let keys_cstr = CString::new(keys).unwrap();
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;
        let options = 0;

        let ret = unsafe {
            ffi::lyd_new_list2(
                self.raw(),
                module
                    .map(|module| module.as_raw())
                    .unwrap_or(std::ptr::null_mut()),
                name_cstr.as_ptr(),
                keys_cstr.as_ptr(),
                options,
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(unsafe { DataNodeRef::from_raw(self.tree, rnode) })
    }

    /// Create a new list node in the data tree.
    ///
    /// The `keys` parameter should be a slice of strings representing the key
    /// values for the new list instance. All keys must be provided in the
    /// correct order.
    ///
    /// Returns the created node.
    pub fn new_list2(
        &mut self,
        module: Option<&SchemaModule<'_>>,
        name: &str,
        keys: &[impl AsRef<str>],
    ) -> Result<DataNodeRef<'a>> {
        let name_cstr = CString::new(name).unwrap();
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;
        let options = 0;

        // Convert keys to raw pointers.
        let keys: Vec<CString> = keys
            .iter()
            .map(|key| CString::new(key.as_ref()).unwrap())
            .collect();
        let mut keys: Vec<*const c_char> =
            keys.iter().map(|key| key.as_ptr()).collect();

        let ret = unsafe {
            ffi::lyd_new_list3(
                self.raw(),
                module
                    .map(|module| module.as_raw())
                    .unwrap_or(std::ptr::null_mut()),
                name_cstr.as_ptr(),
                keys.as_mut_ptr(),
                std::ptr::null_mut(),
                options,
                rnode_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(unsafe { DataNodeRef::from_raw(self.tree, rnode) })
    }

    /// Create a new term node in the data tree.
    pub fn new_term(
        &mut self,
        module: Option<&SchemaModule<'_>>,
        name: &str,
        value: Option<&str>,
    ) -> Result<()> {
        let name_cstr = CString::new(name).unwrap();
        let value_cstr;
        let options = 0;

        let value_ptr = match value {
            Some(value) => {
                value_cstr = CString::new(value).unwrap();
                value_cstr.as_ptr()
            }
            None => std::ptr::null(),
        };

        let ret = unsafe {
            ffi::lyd_new_term(
                self.raw(),
                module
                    .map(|module| module.as_raw())
                    .unwrap_or(std::ptr::null_mut()),
                name_cstr.as_ptr(),
                value_ptr,
                options,
                std::ptr::null_mut(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        Ok(())
    }

    /// Remove the data node.
    pub fn remove(&mut self) {
        unsafe { ffi::lyd_unlink_tree(self.raw()) };
        unsafe { ffi::lyd_free_tree(self.raw()) };
    }
}

impl<'a> Data<'a> for DataNodeRef<'a> {
    fn tree(&self) -> &DataTree<'a> {
        self.tree
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.raw
    }
}

unsafe impl<'a> Binding<'a> for DataNodeRef<'a> {
    type CType = ffi::lyd_node;
    type Container = DataTree<'a>;

    unsafe fn from_raw(
        tree: &'a DataTree<'a>,
        raw: *mut ffi::lyd_node,
    ) -> DataNodeRef<'a> {
        DataNodeRef { tree, raw }
    }
}

impl<'a> NodeIterable<'a> for DataNodeRef<'a> {
    fn parent(&self) -> Option<DataNodeRef<'a>> {
        // NOTE: can't use lyd_parent() since it's an inline function.
        if self.raw.is_null() {
            return None;
        }
        let parent_inner = unsafe { (*self.raw).parent };
        if parent_inner.is_null() {
            return None;
        }
        let rparent = unsafe { &mut (*parent_inner).__bindgen_anon_1.node };
        unsafe { DataNodeRef::from_raw_opt(self.tree, rparent) }
    }

    fn next_sibling(&self) -> Option<DataNodeRef<'a>> {
        let rsibling = unsafe { (*self.raw).next };
        unsafe { DataNodeRef::from_raw_opt(self.tree, rsibling) }
    }

    fn first_child(&self) -> Option<DataNodeRef<'a>> {
        // NOTE: can't use lyd_child() since it's an inline function.
        let snode = unsafe { (*self.raw).schema };
        if snode.is_null() {
            let ropaq = self.raw as *mut ffi::lyd_node_opaq;
            let rchild = unsafe { (*ropaq).child };
            return unsafe { DataNodeRef::from_raw_opt(self.tree, rchild) };
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
        unsafe { DataNodeRef::from_raw_opt(self.tree, rchild) }
    }
}

impl PartialEq for DataNodeRef<'_> {
    fn eq(&self, other: &DataNodeRef<'_>) -> bool {
        self.raw == other.raw
    }
}

unsafe impl Send for DataNodeRef<'_> {}
unsafe impl Sync for DataNodeRef<'_> {}

// ===== impl Metadata =====

impl<'a> Metadata<'a> {
    /// Returns a mutable raw pointer to the underlying C library representation
    /// of the data element metadata.
    pub fn as_raw(&self) -> *mut ffi::lyd_meta {
        self.raw
    }

    /// Metadata name.
    pub fn name(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).name })
    }

    /// Metadata value representation.
    pub fn value(&self) -> &str {
        let rvalue = unsafe { (*self.raw).value };
        let mut canonical = rvalue._canonical;
        if canonical.is_null() {
            canonical = unsafe {
                ffi::lyd_value_get_canonical(
                    self.dnode.tree.context.raw,
                    &rvalue,
                )
            };
        }
        char_ptr_to_str(canonical)
    }

    /// Next metadata.
    #[doc(hidden)]
    pub(crate) fn next(&self) -> Option<Metadata<'a>> {
        let rnext = unsafe { (*self.raw).next };
        unsafe { Metadata::from_raw_opt(self.dnode, rnext) }
    }
}

unsafe impl<'a> Binding<'a> for Metadata<'a> {
    type CType = ffi::lyd_meta;
    type Container = DataNodeRef<'a>;

    unsafe fn from_raw(
        dnode: &'a DataNodeRef<'a>,
        raw: *mut ffi::lyd_meta,
    ) -> Metadata<'a> {
        Metadata { dnode, raw }
    }
}

impl PartialEq for Metadata<'_> {
    fn eq(&self, other: &Metadata<'_>) -> bool {
        self.raw == other.raw
    }
}

unsafe impl Send for Metadata<'_> {}
unsafe impl Sync for Metadata<'_> {}

// ===== impl DataDiff =====

impl<'a> DataDiff<'a> {
    /// Parse (and validate) input data as a YANG data diff.
    pub fn parse_string(
        context: &'a Context,
        data: impl AsRef<[u8]>,
        format: DataFormat,
        parser_options: DataParserFlags,
        validation_options: DataValidationFlags,
    ) -> Result<DataDiff<'a>> {
        let dtree = DataTree::parse_string(
            context,
            data,
            format,
            parser_options,
            validation_options,
        )?;

        Ok(DataDiff { tree: dtree })
    }

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
    pub fn reverse(&self) -> Result<DataDiff<'a>> {
        let mut rnode = std::ptr::null_mut();
        let rnode_ptr = &mut rnode;

        let ret =
            unsafe { ffi::lyd_diff_reverse_all(self.tree.raw, rnode_ptr) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.tree.context));
        }

        Ok(DataDiff {
            tree: unsafe { DataTree::from_raw(self.tree.context, rnode) },
        })
    }
}

impl<'a> Data<'a> for DataDiff<'a> {
    fn tree(&self) -> &DataTree<'a> {
        &self.tree
    }

    fn raw(&self) -> *mut ffi::lyd_node {
        self.tree.raw
    }
}
