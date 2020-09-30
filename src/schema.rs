//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! YANG schema data.

use bitflags::bitflags;
use std::ffi::CString;
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
#[repr(u32)]
pub enum SchemaInputFormat {
    YANG = ffi::LYS_INFORMAT::LYS_IN_YANG,
    YIN = ffi::LYS_INFORMAT::LYS_IN_YIN,
}

/// Schema output formats accepted by libyang.
#[repr(u32)]
pub enum SchemaOutputFormat {
    YANG = ffi::LYS_OUTFORMAT::LYS_OUT_YANG,
    YIN = ffi::LYS_OUTFORMAT::LYS_OUT_YIN,
    TREE = ffi::LYS_OUTFORMAT::LYS_OUT_TREE,
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
    kind: SchemaNodeKind<'a>,
}

/// YANG schema node kind.
#[derive(Clone, Debug)]
pub enum SchemaNodeKind<'a> {
    Container(SchemaNodeContainer<'a>),
    Case(SchemaNodeCase<'a>),
    Choice(SchemaNodeChoice<'a>),
    Leaf(SchemaNodeLeaf<'a>),
    LeafList(SchemaNodeLeafList<'a>),
    List(SchemaNodeList<'a>),
    AnyData(SchemaNodeAnyData<'a>),
    Rpc(SchemaNodeRpc<'a>),
    Action(SchemaNodeAction<'a>),
    Notification(SchemaNodeNotification<'a>),
}

/// YANG container schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeContainer<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_container,
}

/// YANG case schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeCase<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_case,
}

/// YANG choice schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeChoice<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_choice,
}

/// YANG leaf schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeLeaf<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_leaf,
}

/// YANG leaf-list schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeLeafList<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_leaflist,
}

/// YANG list schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeList<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_list,
}

/// YANG anydata schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeAnyData<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_node_anydata,
}

/// YANG RPC schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeRpc<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_action,
}

/// YANG action schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeAction<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_action,
}

/// YANG notification schema node.
#[derive(Clone, Debug)]
pub struct SchemaNodeNotification<'a> {
    context: &'a Context,
    raw: *mut ffi::lysc_notif,
}

/// Methods common to all schema node types.
pub trait SchemaNodeCommon {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node;

    #[doc(hidden)]
    fn check_flag(&self, flag: u32) -> bool {
        let flags = unsafe { (*self.raw()).flags } as u32;
        flags & flag != 0
    }

    /// Context of the schema node.
    fn context(&self) -> &Context;

    /// Schema node module.
    fn module(&self) -> SchemaModule {
        let module = unsafe { (*self.raw()).module };
        SchemaModule::from_raw(self.context(), module)
    }

    /// Schema node name.
    fn name(&self) -> &str {
        char_ptr_to_str(unsafe { (*self.raw()).name })
    }

    /// Description statement.
    fn description(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw()).dsc })
    }

    /// Reference statement.
    fn reference(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw()).ref_ })
    }

    // TODO: list of if-feature expressions.

    /// Generate path of the given node.
    fn path(&self) -> Result<String> {
        let mut buf: [std::os::raw::c_char; 1024] = [0; 1024];

        let pathtype = ffi::LYSC_PATH_TYPE::LYSC_PATH_LOG;
        let ret = unsafe {
            ffi::lysc_path(
                self.raw(),
                pathtype,
                buf.as_mut_ptr(),
                buf.len() as u64,
            )
        };
        if ret.is_null() {
            return Err(Error::new(self.context()));
        }

        Ok(char_ptr_to_string(buf.as_ptr()))
    }

    /// Evaluate an xpath expression on schema nodes.
    fn find(&self, xpath: &str) -> Result<Set<SchemaNode>> {
        let xpath = CString::new(xpath).unwrap();
        let mut set = std::ptr::null_mut();
        let set_ptr = &mut set;
        let options = 0u32;

        let ret = unsafe {
            ffi::lys_find_xpath(self.raw(), xpath.as_ptr(), options, set_ptr)
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(&self.context()));
        }

        let rnodes_count = unsafe { (*set).count } as usize;
        let slice = if rnodes_count == 0 {
            &[]
        } else {
            let rnodes = unsafe { (*set).__bindgen_anon_1.snodes };
            unsafe { slice::from_raw_parts(rnodes, rnodes_count) }
        };

        Ok(Set::new(self.context(), slice))
    }

    /// Evaluate an xpath expression on schema nodes. Return an error if more
    /// than one schema node satisfies the given xpath expression.
    fn find_single(&self, xpath: &str) -> Result<SchemaNode> {
        let mut snodes = self.find(xpath)?;

        // Get first element from the iterator.
        let snode = snodes.next();

        match snode {
            // Error: more that one node satisfies the xpath query.
            Some(_) if snodes.next().is_some() => Err(Error {
                errcode: ffi::LY_ERR::LY_ENOTFOUND,
                msg: Some(
                    "Path refers to more than one schema node".to_string(),
                ),
                path: Some(xpath.to_string()),
                apptag: None,
            }),
            // Success case.
            Some(snode) => Ok(snode),
            // Error: node not found.
            None => Err(Error {
                errcode: ffi::LY_ERR::LY_ENOTFOUND,
                msg: Some("Schema node not found".to_string()),
                path: Some(xpath.to_string()),
                apptag: None,
            }),
        }
    }

    /// Set a schema private pointer to a user pointer.
    ///
    /// Returns previous private pointer when set.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided pointer is valid.
    unsafe fn set_private(
        &self,
        ptr: *mut std::ffi::c_void,
    ) -> Result<Option<*const std::ffi::c_void>> {
        let mut prev = std::ptr::null_mut();
        let prev_ptr = &mut prev;

        let ret = ffi::lysc_set_private(self.raw(), ptr, prev_ptr);
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self.context()));
        }

        if prev.is_null() {
            Ok(None)
        } else {
            Ok(Some(prev))
        }
    }

    /// Get private user data, not used by libyang.
    fn get_private(&self) -> Option<*mut std::ffi::c_void> {
        let priv_ = unsafe { (*self.raw()).priv_ };
        if priv_.is_null() {
            None
        } else {
            Some(priv_)
        }
    }
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
    pub fn rpcs(&self) -> Array<SchemaNodeRpc> {
        let compiled = unsafe { (*self.raw).compiled };
        let rpcs = if compiled.is_null() {
            std::ptr::null()
        } else {
            unsafe { (*compiled).rpcs }
        };
        Array::new(&self.context, rpcs as *mut _)
    }

    /// Returns an iterator over the list of notifications.
    pub fn notifications(&self) -> Array<SchemaNodeNotification> {
        let compiled = unsafe { (*self.raw).compiled };
        let notifications = if compiled.is_null() {
            std::ptr::null()
        } else {
            unsafe { (*compiled).notifs }
        };
        Array::new(&self.context, notifications as *mut _)
    }

    /// Returns an iterator over all data nodes in the schema module
    /// (depth-first search algorithm).
    ///
    /// NOTE: augmentations (from other modules or from the module itself) are
    /// also iterated over.
    pub fn traverse(&'a self) -> impl Iterator<Item = SchemaNode<'a>> {
        self.data().flat_map(|snode| snode.traverse())
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

// ===== impl SchemaNode =====

impl<'a> SchemaNode<'a> {
    /// Returns the kind of this schema node.
    pub fn kind(&self) -> &SchemaNodeKind {
        &self.kind
    }

    /// Returns an iterator over the ancestor schema nodes.
    pub fn ancestors(&self) -> Ancestors<'a, SchemaNode<'a>> {
        let parent = self.parent();
        Ancestors::new(parent)
    }

    /// Returns an iterator over the sibling schema nodes.
    pub fn siblings(&self) -> Siblings<'a, SchemaNode<'a>> {
        let sibling = self.next_sibling();
        Siblings::new(sibling)
    }

    /// Returns an iterator over the child schema nodes.
    pub fn children(&self) -> Siblings<'a, SchemaNode<'a>> {
        let child = self.first_child();
        Siblings::new(child)
    }

    /// Returns an iterator over all elements in the schema tree (depth-first
    /// search algorithm).
    pub fn traverse(self) -> Traverse<'a, SchemaNode<'a>> {
        Traverse::new(self)
    }
}

impl<'a> SchemaNodeCommon for SchemaNode<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

impl<'a> Binding<'a> for SchemaNode<'a> {
    type CType = ffi::lysc_node;
    type Container = Context;

    fn from_raw(context: &'a Context, raw: *mut ffi::lysc_node) -> SchemaNode {
        let nodetype = unsafe { (*raw).nodetype } as u32;
        let kind = match nodetype {
            ffi::LYS_CONTAINER => {
                SchemaNodeKind::Container(SchemaNodeContainer {
                    context,
                    raw: raw as *mut _,
                })
            }
            ffi::LYS_CASE => SchemaNodeKind::Case(SchemaNodeCase {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_CHOICE => SchemaNodeKind::Choice(SchemaNodeChoice {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_LEAF => SchemaNodeKind::Leaf(SchemaNodeLeaf {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_LEAFLIST => SchemaNodeKind::LeafList(SchemaNodeLeafList {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_LIST => SchemaNodeKind::List(SchemaNodeList {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_ANYDATA => SchemaNodeKind::AnyData(SchemaNodeAnyData {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_ACTION => SchemaNodeKind::Action(SchemaNodeAction {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_RPC => SchemaNodeKind::Rpc(SchemaNodeRpc {
                context,
                raw: raw as *mut _,
            }),
            ffi::LYS_NOTIF => {
                SchemaNodeKind::Notification(SchemaNodeNotification {
                    context,
                    raw: raw as *mut _,
                })
            }
            _ => panic!("unknown node type"),
        };
        SchemaNode { context, raw, kind }
    }
}

impl<'a> NodeIterable<'a> for SchemaNode<'a> {
    fn parent(&self) -> Option<SchemaNode<'a>> {
        let parent = unsafe { (&*self.raw).parent };
        if parent.is_null() {
            None
        } else {
            Some(SchemaNode::from_raw(&self.context, parent))
        }
    }

    fn next_sibling(&self) -> Option<SchemaNode<'a>> {
        match self.kind {
            SchemaNodeKind::Container(_)
            | SchemaNodeKind::Case(_)
            | SchemaNodeKind::Choice(_)
            | SchemaNodeKind::Leaf(_)
            | SchemaNodeKind::LeafList(_)
            | SchemaNodeKind::List(_)
            | SchemaNodeKind::AnyData(_) => {
                let next = unsafe { (&*self.raw).next };
                SchemaNode::from_raw_opt(&self.context, next)
            }
            SchemaNodeKind::Rpc(_)
            | SchemaNodeKind::Action(_)
            | SchemaNodeKind::Notification(_) => None,
        }
    }

    fn first_child(&self) -> Option<SchemaNode<'a>> {
        let rchild = unsafe { ffi::lysc_node_children(&*self.raw, 0) };
        SchemaNode::from_raw_opt(&self.context, rchild as *mut _)
    }
}

impl<'a> PartialEq for SchemaNode<'a> {
    fn eq(&self, other: &SchemaNode) -> bool {
        self.raw == other.raw
    }
}

// ===== impl SchemaNodeContainer =====

impl<'a> SchemaNodeContainer<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Returns whether this is a mandatory node.
    pub fn mandatory(&self) -> bool {
        self.check_flag(ffi::LYS_MAND_TRUE)
    }

    /// Returns whether this is a presence container.
    pub fn presence(&self) -> bool {
        self.check_flag(ffi::LYS_PRESENCE)
    }

    /// Array of must restrictions.
    pub fn musts(&self) -> Array<SchemaStmtMust> {
        let array = unsafe { (*self.raw).musts };
        Array::new(&self.context, array)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }

    /// Array of actions.
    pub fn actions(&self) -> Array<SchemaNodeAction> {
        let array = unsafe { (*self.raw).actions };
        Array::new(&self.context, array)
    }

    /// Array of notifications.
    pub fn notifications(&self) -> Array<SchemaNodeNotification> {
        let array = unsafe { (*self.raw).notifs };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeContainer<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeCase =====

impl<'a> SchemaNodeCase<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeCase<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeChoice =====

impl<'a> SchemaNodeChoice<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Returns whether this is a mandatory node.
    pub fn mandatory(&self) -> bool {
        self.check_flag(ffi::LYS_MAND_TRUE)
    }

    /// The default case of the choice.
    pub fn default(&self) -> Option<SchemaNode> {
        let default = unsafe { (*self.raw).dflt } as *mut _;
        SchemaNode::from_raw_opt(&self.context, default)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeChoice<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeLeaf =====

impl<'a> SchemaNodeLeaf<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Returns whether this is a mandatory node.
    pub fn mandatory(&self) -> bool {
        self.check_flag(ffi::LYS_MAND_TRUE)
    }

    /// Returns whether this leaf is a key of a list.
    pub fn key(&self) -> bool {
        self.check_flag(ffi::LYS_KEY)
    }

    /// Returns whether a default value is set.
    pub fn has_default(&self) -> bool {
        self.check_flag(ffi::LYS_SET_DFLT)
    }

    /// Default value.
    pub fn default(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*(*self.raw).dflt).canonical })
    }

    /// Resolved base type.
    pub fn base_type(&self) -> ffi::LY_DATA_TYPE::Type {
        unsafe { (*(*self.raw).type_).basetype }
    }

    /// Units of the leaf's type.
    pub fn units(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).units })
    }

    /// Array of must restrictions.
    pub fn musts(&self) -> Array<SchemaStmtMust> {
        let raw = unsafe { (*self.raw).musts };
        Array::new(&self.context, raw)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeLeaf<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeLeafList =====

impl<'a> SchemaNodeLeafList<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Returns whether this is a mandatory node.
    pub fn mandatory(&self) -> bool {
        self.check_flag(ffi::LYS_MAND_TRUE)
    }

    /// Examine whether the leaf-list is user-ordered.
    pub fn user_ordered(&self) -> bool {
        self.check_flag(ffi::LYS_ORDBY_USER)
    }

    /// Returns whether a default value is set.
    pub fn has_default(&self) -> bool {
        self.check_flag(ffi::LYS_SET_DFLT)
    }

    // TODO: list of default values.

    /// Resolved base type.
    pub fn base_type(&self) -> ffi::LY_DATA_TYPE::Type {
        unsafe { (*(*self.raw).type_).basetype }
    }

    /// Units of the leaf's type.
    pub fn units(&self) -> Option<&str> {
        char_ptr_to_opt_str(unsafe { (*self.raw).units })
    }

    /// The min-elements constraint.
    pub fn min_elements(&self) -> Option<u32> {
        let min = unsafe { (*self.raw).min };
        if min != 0 {
            Some(min)
        } else {
            None
        }
    }

    /// The max-elements constraint.
    pub fn max_elements(&self) -> Option<u32> {
        let max = unsafe { (*self.raw).max };
        if max != std::u32::MAX {
            Some(max)
        } else {
            None
        }
    }

    /// Array of must restrictions.
    pub fn musts(&self) -> Array<SchemaStmtMust> {
        let raw = unsafe { (*self.raw).musts };
        Array::new(&self.context, raw)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeLeafList<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeList =====

impl<'a> SchemaNodeList<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Returns whether this is a mandatory node.
    pub fn mandatory(&self) -> bool {
        self.check_flag(ffi::LYS_MAND_TRUE)
    }

    /// Returns whether this is a keyless list.
    pub fn keyless(&self) -> bool {
        self.check_flag(ffi::LYS_KEYLESS)
    }

    /// Examine whether the list is user-ordered.
    pub fn user_ordered(&self) -> bool {
        self.check_flag(ffi::LYS_ORDBY_USER)
    }

    /// The min-elements constraint.
    pub fn min_elements(&self) -> Option<u32> {
        let min = unsafe { (*self.raw).min };
        if min != 0 {
            Some(min)
        } else {
            None
        }
    }

    /// The max-elements constraint.
    pub fn max_elements(&self) -> Option<u32> {
        let max = unsafe { (*self.raw).max };
        if max != std::u32::MAX {
            Some(max)
        } else {
            None
        }
    }

    /// Array of must restrictions.
    pub fn musts(&self) -> Array<SchemaStmtMust> {
        let raw = unsafe { (*self.raw).musts };
        Array::new(&self.context, raw)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }

    /// Array of actions.
    pub fn actions(&self) -> Array<SchemaNodeAction> {
        let array = unsafe { (*self.raw).actions };
        Array::new(&self.context, array)
    }

    /// Array of notifications.
    pub fn notifications(&self) -> Array<SchemaNodeNotification> {
        let array = unsafe { (*self.raw).notifs };
        Array::new(&self.context, array)
    }

    // TODO: Array of unique nodes.
}

impl<'a> SchemaNodeCommon for SchemaNodeList<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeAnyData =====

impl<'a> SchemaNodeAnyData<'a> {
    /// Returns whether this is a configuration node.
    pub fn config(&self) -> bool {
        self.check_flag(ffi::LYS_CONFIG_W)
    }

    /// Returns whether this is a mandatory node.
    pub fn mandatory(&self) -> bool {
        self.check_flag(ffi::LYS_MAND_TRUE)
    }

    /// Array of must restrictions.
    pub fn musts(&self) -> Array<SchemaStmtMust> {
        let raw = unsafe { (*self.raw).musts };
        Array::new(&self.context, raw)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeAnyData<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

// ===== impl SchemaNodeRpc =====

impl<'a> SchemaNodeRpc<'a> {
    /// RPC's input. Returns a tuple containing the following:
    /// * Iterator over the input child nodes;
    /// * Input's list of must restrictions.
    pub fn input(&self) -> (Siblings<SchemaNode>, Array<SchemaStmtMust>) {
        let input = unsafe { (*self.raw).input };
        let rnode = input.data;
        let rmusts = input.musts;

        let node = SchemaNode::from_raw_opt(&self.context, rnode);
        let nodes = Siblings::new(node);
        let musts = Array::new(&self.context, rmusts);
        (nodes, musts)
    }

    /// RPC's output. Returns a tuple containing the following:
    /// * Iterator over the output child nodes;
    /// * Output's list of must restrictions.
    pub fn output(&self) -> (Siblings<SchemaNode>, Array<SchemaStmtMust>) {
        let output = unsafe { (*self.raw).output };
        let rnode = output.data;
        let rmusts = output.musts;

        let node = SchemaNode::from_raw_opt(&self.context, rnode);
        let nodes = Siblings::new(node);
        let musts = Array::new(&self.context, rmusts);
        (nodes, musts)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeRpc<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

impl<'a> Binding<'a> for SchemaNodeRpc<'a> {
    type CType = ffi::lysc_action;
    type Container = Context;

    fn from_raw(
        context: &'a Context,
        raw: *mut ffi::lysc_action,
    ) -> SchemaNodeRpc {
        SchemaNodeRpc { context, raw }
    }
}

// ===== impl SchemaNodeAction =====

impl<'a> SchemaNodeAction<'a> {
    /// Action's input. Returns a tuple containing the following:
    /// * First input child node (linked list);
    /// * Input's list of must restrictions.
    pub fn input(&self) -> (Siblings<SchemaNode>, Array<SchemaStmtMust>) {
        let input = unsafe { (*self.raw).input };
        let rnode = input.data;
        let rmusts = input.musts;

        let node = SchemaNode::from_raw_opt(&self.context, rnode);
        let nodes = Siblings::new(node);
        let musts = Array::new(&self.context, rmusts);
        (nodes, musts)
    }

    /// Action's output. Returns a tuple containing the following:
    /// * First output child node (linked list);
    /// * Output's list of must restrictions.
    pub fn output(&self) -> (Siblings<SchemaNode>, Array<SchemaStmtMust>) {
        let output = unsafe { (*self.raw).output };
        let rnode = output.data;
        let rmusts = output.musts;

        let node = SchemaNode::from_raw_opt(&self.context, rnode);
        let nodes = Siblings::new(node);
        let musts = Array::new(&self.context, rmusts);
        (nodes, musts)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeAction<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

impl<'a> Binding<'a> for SchemaNodeAction<'a> {
    type CType = ffi::lysc_action;
    type Container = Context;

    fn from_raw(
        context: &'a Context,
        raw: *mut ffi::lysc_action,
    ) -> SchemaNodeAction {
        SchemaNodeAction { context, raw }
    }
}

// ===== impl SchemaNodeNotification =====

impl<'a> SchemaNodeNotification<'a> {
    /// Array of must restrictions.
    pub fn musts(&self) -> Array<SchemaStmtMust> {
        let raw = unsafe { (*self.raw).musts };
        Array::new(&self.context, raw)
    }

    /// Array of when statements.
    pub fn whens(&self) -> Array<SchemaStmtWhen> {
        let array = unsafe { (*self.raw).when };
        Array::new(&self.context, array)
    }
}

impl<'a> SchemaNodeCommon for SchemaNodeNotification<'a> {
    #[doc(hidden)]
    fn raw(&self) -> *mut ffi::lysc_node {
        self.raw as *mut _
    }

    fn context(&self) -> &Context {
        &self.context
    }
}

impl<'a> Binding<'a> for SchemaNodeNotification<'a> {
    type CType = ffi::lysc_notif;
    type Container = Context;

    fn from_raw(
        context: &'a Context,
        raw: *mut ffi::lysc_notif,
    ) -> SchemaNodeNotification {
        SchemaNodeNotification { context, raw }
    }
}

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
