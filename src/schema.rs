//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! YANG schema data.

use bitflags::bitflags;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::os::unix::io::AsRawFd;
use std::slice;

use crate::context::Context;
use crate::error::{Error, Result};
use crate::iter::{Ancestors, Array, NodeIterable, Set, Siblings, Traverse};
use crate::utils::*;
use libyang2_sys as ffi;

/// Available YANG schema tree structures representing YANG module.
#[derive(Clone, Debug)]
pub struct SchemaModule<'a> {
    context: &'a Context,
    raw: *mut ffi::lys_module,
}

/// Schema input formats accepted by libyang.
#[allow(clippy::upper_case_acronyms)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SchemaInputFormat {
    YANG = ffi::LYS_INFORMAT::LYS_IN_YANG,
    YIN = ffi::LYS_INFORMAT::LYS_IN_YIN,
}

/// Schema output formats accepted by libyang.
#[allow(clippy::upper_case_acronyms)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SchemaOutputFormat {
    YANG = ffi::LYS_OUTFORMAT::LYS_OUT_YANG,
    YIN = ffi::LYS_OUTFORMAT::LYS_OUT_YIN,
    TREE = ffi::LYS_OUTFORMAT::LYS_OUT_TREE,
}

/// Schema path format.
#[allow(clippy::upper_case_acronyms)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SchemaPathFormat {
    /// Descriptive path format used in log messages.
    LOG = ffi::LYSC_PATH_TYPE::LYSC_PATH_LOG,
    /// Similar to LOG except that schema-only nodes (choice, case) are
    /// skipped.
    DATA = ffi::LYSC_PATH_TYPE::LYSC_PATH_DATA,
}

bitflags! {
    /// Schema printer flags.
    pub struct SchemaPrinterFlags: u32 {
        /// Flag for output without indentation and formatting new lines.
        const SHRINK = ffi::LYS_PRINT_SHRINK;
        /// Print only top-level/reference node information, do not print
        /// information from the substatements.
        const NO_SUBSTMT = ffi::LYS_PRINT_NO_SUBSTMT;
    }
}

/// Generic YANG schema node.
#[derive(Clone, Debug)]
pub struct SchemaNode<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node,
    kind: SchemaNodeKind,
}

/// YANG schema node kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SchemaNodeKind {
    Container,
    Case,
    Choice,
    Leaf,
    LeafList,
    List,
    AnyData,
    Rpc,
    Input,
    Output,
    Action,
    Notification,
}

/// YANG must substatement.
#[derive(Clone, Debug)]
pub struct SchemaStmtMust<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_must,
}

/// YANG when substatement.
#[derive(Clone, Debug)]
pub struct SchemaStmtWhen<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_when,
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub enum DataValueType {
    Unknown = 0,
    Binary = 1,
    Uint8 = 2,
    Uint16 = 3,
    Uint32 = 4,
    Uint64 = 5,
    String = 6,
    Bits = 7,
    Bool = 8,
    Dec64 = 9,
    Empty = 10,
    Enum = 11,
    IdentityRef = 12,
    InstanceId = 13,
    LeafRef = 14,
    Union = 15,
    Int8 = 16,
    Int16 = 17,
    Int32 = 18,
    Int64 = 19,
}

// ===== impl SchemaModule =====

impl<'a> SchemaModule<'a> {
    /// Name of the module.
    pub fn name(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).name })
    }

    /// Revision of the module.
    pub fn revision(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).revision })
    }

    /// Namespace of the module.
    pub fn namespace(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).ns })
    }

    /// Prefix of the module.
    pub fn prefix(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).prefix })
    }

    /// File path, if the schema was read from a file.
    pub fn filepath(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).filepath })
    }

    /// Party/company responsible for the module.
    pub fn organization(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).org })
    }

    /// Contact information for the module.
    pub fn contact(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).contact })
    }

    /// Description of the module.
    pub fn description(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).dsc })
    }

    /// Cross-reference for the module.
    pub fn reference(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).ref_ })
    }

    /// Make the specific module implemented.
    pub fn set_implemented(&self) -> Result<()> {
        let ret =
            unsafe { ffi::lys_set_implemented(self.raw, std::ptr::null_mut()) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(())
    }

    /// Return true if the module is implemented, not just imported.
    pub fn is_implemented(&self) -> bool {
        unsafe { (*self.raw).implemented != 0 }
    }

    /// Print schema tree in the specified format into a file descriptor.
    pub fn print_file<F: AsRawFd>(
        &self,
        fd: F,
        format: SchemaOutputFormat,
        options: SchemaPrinterFlags,
    ) -> Result<()> {
        let ret = unsafe {
            ffi::lys_print_fd(
                fd.as_raw_fd(),
                self.raw,
                format as u32,
                options.bits(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(())
    }

    /// Print schema tree in the specified format into a string.
    pub fn print_string(
        &self,
        format: SchemaOutputFormat,
        options: SchemaPrinterFlags,
    ) -> Result<String> {
        let mut cstr = std::ptr::null_mut();
        let cstr_ptr = &mut cstr;

        let ret = unsafe {
            ffi::lys_print_mem(
                cstr_ptr,
                self.raw,
                format as u32,
                options.bits(),
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        Ok(char_ptr_to_string(cstr))
    }

    /// Returns an iterator over the top-level data nodes.
    pub fn data(&self) -> Siblings<'a, SchemaNode<'a>> {
        let compiled = unsafe { (*self.raw).compiled };
        let rdata = if compiled.is_null() {
            std::ptr::null()
        } else {
            unsafe { (*compiled).data }
        };
        let data = SchemaNode::from_raw_opt(&self.context, rdata as *mut _);
        Siblings::new(data)
    }

    /// Returns an iterator over the list of RPCs.
    pub fn rpcs(&self) -> Siblings<'a, SchemaNode<'a>> {
        let compiled = unsafe { (*self.raw).compiled };
        let rdata = if compiled.is_null() {
            std::ptr::null()
        } else {
            unsafe { (*compiled).rpcs }
        };
        let rpcs = SchemaNode::from_raw_opt(&self.context, rdata as *mut _);
        Siblings::new(rpcs)
    }

    /// Returns an iterator over the list of notifications.
    pub fn notifications(&self) -> Siblings<'a, SchemaNode<'a>> {
        let compiled = unsafe { (*self.raw).compiled };
        let rdata = if compiled.is_null() {
            std::ptr::null()
        } else {
            unsafe { (*compiled).notifs }
        };
        let notifications =
            SchemaNode::from_raw_opt(&self.context, rdata as *mut _);
        Siblings::new(notifications)
    }

    /// Returns an iterator over all data nodes in the schema module
    /// (depth-first search algorithm).
    ///
    /// NOTE: augmentations (from other modules or from the module itself) are
    /// also iterated over.
    pub fn traverse(&self) -> impl Iterator<Item = SchemaNode<'a>> {
        let data = self.data().flat_map(|snode| snode.traverse());
        let rpcs = self.rpcs().flat_map(|snode| snode.traverse());
        let notifications =
            self.notifications().flat_map(|snode| snode.traverse());
        data.chain(rpcs).chain(notifications)
    }
}

impl<'a> Binding<'a> for SchemaModule<'a> {
    type CType = ffi::lys_module;
    type Container = Context;

    fn from_raw(
        context: &'a Context,
        raw: *mut ffi::lys_module,
    ) -> SchemaModule {
        SchemaModule { context, raw }
    }
}

impl<'a> PartialEq for SchemaModule<'a> {
    fn eq(&self, other: &SchemaModule) -> bool {
        self.raw == other.raw
    }
}

unsafe impl Send for SchemaModule<'_> {}
unsafe impl Sync for SchemaModule<'_> {}

// ===== impl SchemaNode =====

impl<'a> SchemaNode<'a> {
    #[doc(hidden)]
    fn check_flag(&self, flag: u32) -> bool {
        let flags = unsafe { (*self.raw).flags } as u32;
        flags & flag != 0
    }

    /// Schema node module.
    pub fn module(&self) -> SchemaModule {
        let module = unsafe { (*self.raw).module };
        SchemaModule::from_raw(self.context, module)
    }

    /// Returns the kind of the schema node.
    pub fn kind(&self) -> SchemaNodeKind {
        self.kind
    }

    /// Schema node name.
    pub fn name(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw).name })
    }

    /// Description statement.
    pub fn description(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).dsc })
    }

    /// Reference statement.
    pub fn reference(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).ref_ })
    }

    /// Generate path of the node.
    pub fn path(&self, format: SchemaPathFormat) -> String {
        let buf = std::mem::MaybeUninit::<[c_char; 4096]>::uninit();
        let mut buf = unsafe { buf.assume_init() };

        let ret = unsafe {
            ffi::lysc_path(
                self.raw,
                format as u32,
                buf.as_mut_ptr(),
                buf.len() as u64,
            )
        };
        if ret.is_null() {
            panic!("Failed to generate path of the schema node");
        }

        char_ptr_to_string(buf.as_ptr())
    }

    /// Evaluate an xpath expression on the node.
    pub fn find_xpath(&self, xpath: &str) -> Result<Set<SchemaNode>> {
        let xpath = CString::new(xpath).unwrap();
        let mut set = std::ptr::null_mut();
        let set_ptr = &mut set;
        let options = 0u32;

        let ret = unsafe {
            ffi::lys_find_xpath(
                std::ptr::null(),
                self.raw,
                xpath.as_ptr(),
                options,
                set_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context));
        }

        let rnodes_count = unsafe { (*set).count } as usize;
        let slice = if rnodes_count == 0 {
            &[]
        } else {
            let rnodes = unsafe { (*set).__bindgen_anon_1.snodes };
            unsafe { slice::from_raw_parts(rnodes, rnodes_count) }
        };

        Ok(Set::new(self.context, slice))
    }

    /// Get a schema node based on the given data path (JSON format).
    pub fn find_path(&self, path: &str) -> Result<SchemaNode> {
        let path = CString::new(path).unwrap();

        let rnode = unsafe {
            ffi::lys_find_path(std::ptr::null(), self.raw, path.as_ptr(), 0)
        };
        if rnode.is_null() {
            return Err(Error::new(self.context));
        }

        Ok(SchemaNode::from_raw(self.context, rnode as *mut _))
    }

    /// Check type restrictions applicable to the particular leaf/leaf-list with
    /// the given string value.
    pub fn value_validate(&self, value: &str) -> bool {
        let value_len = value.len() as u64;
        let value = CString::new(value).unwrap();
        let ret = unsafe {
            ffi::lyd_value_validate(
                self.context.raw,
                self.raw,
                value.as_ptr(),
                value_len,
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        ret == ffi::LY_ERR::LY_SUCCESS
    }

    /// Returns whether the node is a configuration node.
    pub fn is_config(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Container
            | SchemaNodeKind::Case
            | SchemaNodeKind::Choice
            | SchemaNodeKind::Leaf
            | SchemaNodeKind::LeafList
            | SchemaNodeKind::List
            | SchemaNodeKind::AnyData => self.check_flag(ffi::LYS_CONFIG_W),
            _ => false,
        }
    }

    /// Returns whether the node is mandatory.
    pub fn is_mandatory(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Container
            | SchemaNodeKind::Choice
            | SchemaNodeKind::Leaf
            | SchemaNodeKind::LeafList
            | SchemaNodeKind::List
            | SchemaNodeKind::AnyData => self.check_flag(ffi::LYS_MAND_TRUE),
            _ => false,
        }
    }

    /// Returns whether the node is a non-presence container.
    pub fn is_np_container(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Container => !self.check_flag(ffi::LYS_PRESENCE),
            _ => false,
        }
    }

    /// Returns whether the node is a list's key.
    pub fn is_list_key(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Leaf => self.check_flag(ffi::LYS_KEY),
            _ => false,
        }
    }

    /// Returns whether the node is a keyless list.
    pub fn is_keyless_list(&self) -> bool {
        match self.kind {
            SchemaNodeKind::List => self.check_flag(ffi::LYS_KEYLESS),
            _ => false,
        }
    }

    /// Returns whether the node is an user-ordered list or leaf-list.
    pub fn is_user_ordered(&self) -> bool {
        match self.kind {
            SchemaNodeKind::LeafList | SchemaNodeKind::List => {
                self.check_flag(ffi::LYS_ORDBY_USER)
            }
            _ => false,
        }
    }

    /// Returns whether the node appears only in the schema tree and not in the
    /// data tree.
    pub fn is_schema_only(&self) -> bool {
        matches!(self.kind(), SchemaNodeKind::Choice | SchemaNodeKind::Case)
    }

    /// Returns whether the node is in the subtree of an input statement.
    pub fn is_within_input(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Container
            | SchemaNodeKind::Case
            | SchemaNodeKind::Choice
            | SchemaNodeKind::Leaf
            | SchemaNodeKind::LeafList
            | SchemaNodeKind::List
            | SchemaNodeKind::AnyData => self.check_flag(ffi::LYS_IS_INPUT),
            _ => false,
        }
    }

    /// Returns whether the node is in the subtree of an output statement.
    pub fn is_within_output(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Container
            | SchemaNodeKind::Case
            | SchemaNodeKind::Choice
            | SchemaNodeKind::Leaf
            | SchemaNodeKind::LeafList
            | SchemaNodeKind::List
            | SchemaNodeKind::AnyData => self.check_flag(ffi::LYS_IS_OUTPUT),
            _ => false,
        }
    }

    /// Returns whether the node is in the subtree of a notification statement.
    pub fn is_within_notification(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Container
            | SchemaNodeKind::Case
            | SchemaNodeKind::Choice
            | SchemaNodeKind::Leaf
            | SchemaNodeKind::LeafList
            | SchemaNodeKind::List
            | SchemaNodeKind::AnyData => self.check_flag(ffi::LYS_IS_NOTIF),
            _ => false,
        }
    }

    /// Returns whether a default value is set.
    pub fn has_default(&self) -> bool {
        match self.kind {
            SchemaNodeKind::Case
            | SchemaNodeKind::Leaf
            | SchemaNodeKind::LeafList => self.check_flag(ffi::LYS_SET_DFLT),
            _ => false,
        }
    }

    /// The default value of the leaf.
    pub fn default_value(&self) -> Option<&str> {
        let default = unsafe {
            match self.kind() {
                SchemaNodeKind::Leaf => {
                    (*(*(self.raw as *mut ffi::lysc_node_leaf)).dflt)._canonical
                }
                _ => return None,
            }
        };

        char_ptr_to_opt_str(default)
    }

    /// The default case of the choice.
    pub fn default_case(&self) -> Option<SchemaNode> {
        let default = unsafe {
            match self.kind() {
                SchemaNodeKind::Choice => {
                    (*(self.raw as *mut ffi::lysc_node_choice)).dflt
                }
                _ => return None,
            }
        };

        SchemaNode::from_raw_opt(&self.context, default as *mut _)
    }

    // TODO: list of leaf-list default values.

    /// Resolved base type.
    pub fn base_type(&self) -> Option<DataValueType> {
        let ltype = unsafe {
            match self.kind() {
                SchemaNodeKind::Leaf => {
                    (*(self.raw as *mut ffi::lysc_node_leaf)).type_
                }
                SchemaNodeKind::LeafList => {
                    (*(self.raw as *mut ffi::lysc_node_leaflist)).type_
                }
                _ => return None,
            }
        };
        let ltype = unsafe { (*ltype).basetype };
        Some(DataValueType::from_u32(ltype).unwrap())
    }

    /// Units of the leaf(-list)'s type.
    pub fn units(&self) -> Option<&str> {
        let units = unsafe {
            match self.kind() {
                SchemaNodeKind::Leaf => {
                    (*(self.raw as *mut ffi::lysc_node_leaf)).units
                }
                SchemaNodeKind::LeafList => {
                    (*(self.raw as *mut ffi::lysc_node_leaflist)).units
                }
                _ => return None,
            }
        };

        char_ptr_to_opt_str(units)
    }

    /// The min-elements constraint.
    pub fn min_elements(&self) -> Option<u32> {
        let min = unsafe {
            match self.kind() {
                SchemaNodeKind::LeafList => {
                    (*(self.raw as *mut ffi::lysc_node_leaflist)).min
                }
                SchemaNodeKind::List => {
                    (*(self.raw as *mut ffi::lysc_node_list)).min
                }
                _ => return None,
            }
        };

        if min != 0 {
            Some(min)
        } else {
            None
        }
    }

    /// The max-elements constraint.
    pub fn max_elements(&self) -> Option<u32> {
        let max = unsafe {
            match self.kind() {
                SchemaNodeKind::LeafList => {
                    (*(self.raw as *mut ffi::lysc_node_leaflist)).max
                }
                SchemaNodeKind::List => {
                    (*(self.raw as *mut ffi::lysc_node_list)).max
                }
                _ => return None,
            }
        };
        if max != std::u32::MAX {
            Some(max)
        } else {
            None
        }
    }

    /// Array of must restrictions.
    pub fn musts(&self) -> Option<Array<SchemaStmtMust>> {
        let array = unsafe { ffi::lysc_node_musts(self.raw) };
        let ptr_size = mem::size_of::<ffi::lysc_must>();
        Some(Array::new(&self.context, array as *mut _, ptr_size))
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { ffi::lysc_node_when(self.raw) };
        let ptr_size = mem::size_of::<ffi::lysc_when>();
        Array::new(&self.context, array as *mut _, ptr_size)
    }

    // TODO: list of if-feature expressions.

    /// Array of actions.
    pub fn actions(&self) -> Option<Array<SchemaNode>> {
        let array = unsafe {
            match self.kind {
                SchemaNodeKind::Container => {
                    (*(self.raw as *mut ffi::lysc_node_container)).actions
                }
                SchemaNodeKind::List => {
                    (*(self.raw as *mut ffi::lysc_node_list)).actions
                }
                _ => return None,
            }
        };

        let ptr_size = mem::size_of::<ffi::lysc_node_action>();
        Some(Array::new(&self.context, array as *mut _, ptr_size))
    }

    /// Array of notifications.
    pub fn notifications(&self) -> Option<Array<SchemaNode>> {
        let array = unsafe {
            match self.kind {
                SchemaNodeKind::Container => {
                    (*(self.raw as *mut ffi::lysc_node_container)).notifs
                }
                SchemaNodeKind::List => {
                    (*(self.raw as *mut ffi::lysc_node_list)).notifs
                }
                _ => return None,
            }
        };

        let ptr_size = mem::size_of::<ffi::lysc_node_notif>();
        Some(Array::new(&self.context, array as *mut _, ptr_size))
    }

    /// RPC's input. Returns a tuple containing the following:
    /// * Iterator over the input child nodes;
    /// * Input's list of must restrictions.
    pub fn input(
        &self,
    ) -> Option<(Siblings<SchemaNode>, Array<SchemaStmtMust>)> {
        match self.kind {
            SchemaNodeKind::Rpc | SchemaNodeKind::Action => {
                let raw = self.raw as *mut ffi::lysc_node_action;
                let input = unsafe { (*raw).input };
                let rnode = input.child;
                let rmusts = input.musts;

                let node = SchemaNode::from_raw_opt(&self.context, rnode);
                let nodes = Siblings::new(node);
                let ptr_size = mem::size_of::<ffi::lysc_must>();
                let musts = Array::new(&self.context, rmusts, ptr_size);
                Some((nodes, musts))
            }
            _ => None,
        }
    }

    /// RPC's output. Returns a tuple containing the following:
    /// * Iterator over the output child nodes;
    /// * Output's list of must restrictions.
    pub fn output(
        &self,
    ) -> Option<(Siblings<SchemaNode>, Array<SchemaStmtMust>)> {
        match self.kind {
            SchemaNodeKind::Rpc | SchemaNodeKind::Action => {
                let raw = self.raw as *mut ffi::lysc_node_action;
                let output = unsafe { (*raw).output };
                let rnode = output.child;
                let rmusts = output.musts;

                let node = SchemaNode::from_raw_opt(&self.context, rnode);
                let nodes = Siblings::new(node);
                let ptr_size = mem::size_of::<ffi::lysc_must>();
                let musts = Array::new(&self.context, rmusts, ptr_size);
                Some((nodes, musts))
            }
            _ => None,
        }
    }

    /// Returns an iterator over the ancestor schema nodes.
    pub fn ancestors(&self) -> Ancestors<'a, SchemaNode<'a>> {
        let parent = self.parent();
        Ancestors::new(parent)
    }

    /// Returns an iterator over this schema node and its ancestors.
    pub fn inclusive_ancestors(&self) -> Ancestors<'a, SchemaNode<'a>> {
        Ancestors::new(Some(self.clone()))
    }

    /// Returns an iterator over the sibling schema nodes.
    pub fn siblings(&self) -> Siblings<'a, SchemaNode<'a>> {
        let sibling = self.next_sibling();
        Siblings::new(sibling)
    }

    /// Returns an iterator over this schema node and its siblings.
    pub fn inclusive_siblings(&self) -> Siblings<'a, SchemaNode<'a>> {
        Siblings::new(Some(self.clone()))
    }

    /// Returns an iterator over the child schema nodes.
    pub fn children(&self) -> Siblings<'a, SchemaNode<'a>> {
        let child = self.first_child();
        Siblings::new(child)
    }

    /// Returns an iterator over all elements in the schema tree (depth-first
    /// search algorithm).
    pub fn traverse(&self) -> Traverse<'a, SchemaNode<'a>> {
        Traverse::new(self.clone())
    }

    /// Returns an iterator over the keys of the list.
    pub fn list_keys(&self) -> impl Iterator<Item = SchemaNode<'a>> {
        self.children().filter(|snode| snode.is_list_key())
    }

    /// Set a schema private pointer to a user pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided pointer is valid.
    pub unsafe fn set_private(&self, ptr: *mut c_void) {
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

impl<'a> Binding<'a> for SchemaNode<'a> {
    type CType = ffi::lysc_node;
    type Container = Context;

    fn from_raw(context: &'a Context, raw: *mut ffi::lysc_node) -> SchemaNode {
        let nodetype = unsafe { (*raw).nodetype } as u32;
        let kind = match nodetype {
            ffi::LYS_CONTAINER => SchemaNodeKind::Container,
            ffi::LYS_CASE => SchemaNodeKind::Case,
            ffi::LYS_CHOICE => SchemaNodeKind::Choice,
            ffi::LYS_LEAF => SchemaNodeKind::Leaf,
            ffi::LYS_LEAFLIST => SchemaNodeKind::LeafList,
            ffi::LYS_LIST => SchemaNodeKind::List,
            ffi::LYS_ANYDATA => SchemaNodeKind::AnyData,
            ffi::LYS_ACTION => SchemaNodeKind::Action,
            ffi::LYS_RPC => SchemaNodeKind::Rpc,
            ffi::LYS_INPUT => SchemaNodeKind::Input,
            ffi::LYS_OUTPUT => SchemaNodeKind::Output,
            ffi::LYS_NOTIF => SchemaNodeKind::Notification,
            _ => panic!("unknown node type"),
        };
        SchemaNode { context, raw, kind }
    }
}

impl<'a> NodeIterable<'a> for SchemaNode<'a> {
    fn parent(&self) -> Option<SchemaNode<'a>> {
        let rparent = unsafe { (*self.raw).parent };
        SchemaNode::from_raw_opt(&self.context, rparent)
    }

    fn next_sibling(&self) -> Option<SchemaNode<'a>> {
        let rnext = unsafe { (*self.raw).next };
        SchemaNode::from_raw_opt(&self.context, rnext)
    }

    fn first_child(&self) -> Option<SchemaNode<'a>> {
        let rchild = unsafe { ffi::lysc_node_child(&*self.raw) };
        SchemaNode::from_raw_opt(&self.context, rchild as *mut _)
    }
}

impl<'a> PartialEq for SchemaNode<'a> {
    fn eq(&self, other: &SchemaNode) -> bool {
        self.raw == other.raw
    }
}

unsafe impl Send for SchemaNode<'_> {}
unsafe impl Sync for SchemaNode<'_> {}

// ===== impl SchemaStmtMust =====

impl<'a> SchemaStmtMust<'a> {
    // TODO: XPath condition

    /// description substatement.
    pub fn description(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).dsc })
    }

    /// reference substatement.
    pub fn reference(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).ref_ })
    }

    /// error-message substatement.
    pub fn error_msg(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).emsg })
    }

    /// error-app-tag substatement.
    pub fn error_apptag(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).eapptag })
    }
}

impl<'a> Binding<'a> for SchemaStmtMust<'a> {
    type CType = ffi::lysc_must;
    type Container = Context;

    fn from_raw(
        context: &'a Context,
        raw: *mut ffi::lysc_must,
    ) -> SchemaStmtMust {
        SchemaStmtMust { context, raw }
    }
}

unsafe impl Send for SchemaStmtMust<'_> {}
unsafe impl Sync for SchemaStmtMust<'_> {}

// ===== impl SchemaStmtWhen =====

impl<'a> SchemaStmtWhen<'a> {
    // TODO: XPath condition

    /// description substatement.
    pub fn description(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).dsc })
    }

    /// reference substatement.
    pub fn reference(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).ref_ })
    }
}

impl<'a> Binding<'a> for SchemaStmtWhen<'a> {
    type CType = *mut ffi::lysc_when;
    type Container = Context;

    fn from_raw(
        context: &'a Context,
        raw: *mut *mut ffi::lysc_when,
    ) -> SchemaStmtWhen {
        let raw = unsafe { *raw };
        SchemaStmtWhen { context, raw }
    }
}

unsafe impl Send for SchemaStmtWhen<'_> {}
unsafe impl Sync for SchemaStmtWhen<'_> {}
