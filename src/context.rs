//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! YANG context.

use bitflags::bitflags;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::error::{Error, Result};
use crate::iter::SchemaModules;
use crate::schema::{SchemaModule, SchemaNode};
use crate::utils::Binding;
use libyang2_sys as ffi;

/// Context of the YANG schemas.
///
/// [Official C documentation]
///
/// [Official C documentation]: https://netopeer.liberouter.org/doc/libyang/libyang2/html/howto_context.html
#[derive(Debug)]
pub struct Context {
    pub(crate) raw: *mut ffi::ly_ctx,
}

bitflags! {
    /// Options to change context behavior.
    pub struct ContextFlags: u16 {
        /// All the imported modules of the schema being parsed are implemented.
        const ALL_IMPLEMENTED = ffi::LY_CTX_ALL_IMPLEMENTED as u16;

        /// Implement all imported modules "referenced" from an implemented
        /// module. Normally, leafrefs, augment and deviation targets are
        /// implemented as specified by YANG 1.1. In addition to this, implement
        /// any modules of nodes referenced by when and must conditions and by
        /// any default values. Generally, only if all these modules are
        /// implemented, the explicitly implemented modules can be properly
        /// used and instantiated in data.
        const REF_IMPLEMENTED = ffi::LY_CTX_REF_IMPLEMENTED as u16;

        /// Do not internally implement ietf-yang-library module. This option
        /// cannot be changed on existing context.
        const NO_YANGLIBRARY = ffi::LY_CTX_NO_YANGLIBRARY as u16;

        /// Do not search for schemas in context's searchdirs neither in current
        /// working directory.
        const DISABLE_SEARCHDIRS = ffi::LY_CTX_DISABLE_SEARCHDIRS as u16;

        /// Do not automatically search for schemas in current working
        /// directory, which is by default searched automatically (despite not
        /// recursively).
        const DISABLE_SEARCHDIR_CWD = ffi::LY_CTX_DISABLE_SEARCHDIR_CWD as u16;
    }
}

impl Context {
    /// Create libyang context.
    ///
    /// Context is used to hold all information about schemas. Usually, the
    /// application is supposed to work with a single context in which
    /// libyang is holding all schemas (and other internal information)
    /// according to which the data trees will be processed and validated.
    pub fn new<P: AsRef<Path>>(
        search_dir: P,
        options: ContextFlags,
    ) -> Result<Context> {
        let search_dir =
            CString::new(search_dir.as_ref().as_os_str().as_bytes()).unwrap();
        let mut context = std::ptr::null_mut();
        let ctx_ptr = &mut context;

        let ret = unsafe {
            ffi::ly_ctx_new(search_dir.as_ptr(), options.bits, ctx_ptr)
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            // Need to construct error structure by hand.
            return Err(Error {
                errcode: ret,
                msg: None,
                path: None,
                apptag: None,
            });
        }

        Ok(Context { raw: context })
    }

    /// Add the search path into libyang context.
    pub fn set_searchdir<P: AsRef<Path>>(&self, search_dir: P) -> Result<()> {
        let search_dir =
            CString::new(search_dir.as_ref().as_os_str().as_bytes()).unwrap();
        let ret =
            unsafe { ffi::ly_ctx_set_searchdir(self.raw, search_dir.as_ptr()) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Clean the search path from the libyang context.
    ///
    /// To remove the recently added search path(s), use
    /// Context::unset_searchdir_last().
    pub fn unset_searchdir<P: AsRef<Path>>(&self, search_dir: P) -> Result<()> {
        let search_dir =
            CString::new(search_dir.as_ref().as_os_str().as_bytes()).unwrap();
        let ret = unsafe {
            ffi::ly_ctx_unset_searchdir(self.raw, search_dir.as_ptr())
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Clean all search paths from the libyang context.
    pub fn unset_searchdirs(&self) -> Result<()> {
        let ret =
            unsafe { ffi::ly_ctx_unset_searchdir(self.raw, std::ptr::null()) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Remove the least recently added search path(s) from the libyang context.
    ///
    /// To remove a specific search path by its value, use
    /// Context::unset_searchdir().
    pub fn unset_searchdir_last(&self, count: u32) -> Result<()> {
        let ret = unsafe { ffi::ly_ctx_unset_searchdir_last(self.raw, count) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Get the currently set context's options.
    pub fn get_options(&self) -> ContextFlags {
        let options = unsafe { ffi::ly_ctx_get_options(self.raw) };
        ContextFlags::from_bits_truncate(options)
    }

    /// Set some of the context's options.
    pub fn set_options(&self, options: ContextFlags) -> Result<()> {
        let ret = unsafe { ffi::ly_ctx_set_options(self.raw, options.bits) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Unset some of the context's options.
    pub fn unset_options(&self, options: ContextFlags) -> Result<()> {
        let ret = unsafe { ffi::ly_ctx_unset_options(self.raw, options.bits) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Get current ID of the modules set.
    pub fn get_module_set_id(&self) -> u16 {
        unsafe { ffi::ly_ctx_get_module_set_id(self.raw) }
    }

    /// Get YANG module of the given name and revision.
    ///
    /// If the revision is not specified, the schema with no revision is
    /// returned (if it is present in the context).
    pub fn get_module(
        &self,
        name: &str,
        revision: Option<&str>,
    ) -> Option<SchemaModule> {
        let name = CString::new(name).unwrap();
        let revision_cstr;

        let revision_ptr = match revision {
            Some(revision) => {
                revision_cstr = CString::new(revision).unwrap();
                revision_cstr.as_ptr()
            }
            None => std::ptr::null(),
        };
        let module = unsafe {
            ffi::ly_ctx_get_module(self.raw, name.as_ptr(), revision_ptr)
        };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get the latest revision of the YANG module specified by its name.
    ///
    /// YANG modules with no revision are supposed to be the oldest one.
    pub fn get_module_latest(&self, name: &str) -> Option<SchemaModule> {
        let name = CString::new(name).unwrap();
        let module =
            unsafe { ffi::ly_ctx_get_module_latest(self.raw, name.as_ptr()) };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get the (only) implemented YANG module specified by its name.
    pub fn get_module_implemented(&self, name: &str) -> Option<SchemaModule> {
        let name = CString::new(name).unwrap();
        let module = unsafe {
            ffi::ly_ctx_get_module_implemented(self.raw, name.as_ptr())
        };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// YANG module of the given namespace and revision.
    ///
    /// If the revision is not specified, the schema with no revision is
    /// returned (if it is present in the context).
    pub fn get_module_ns(
        &self,
        ns: &str,
        revision: Option<&str>,
    ) -> Option<SchemaModule> {
        let ns = CString::new(ns).unwrap();
        let revision_cstr;

        let revision_ptr = match revision {
            Some(revision) => {
                revision_cstr = CString::new(revision).unwrap();
                revision_cstr.as_ptr()
            }
            None => std::ptr::null(),
        };

        let module = unsafe {
            ffi::ly_ctx_get_module_ns(self.raw, ns.as_ptr(), revision_ptr)
        };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get the latest revision of the YANG module specified by its namespace.
    ///
    /// YANG modules with no revision are supposed to be the oldest one.
    pub fn get_module_latest_ns(&self, ns: &str) -> Option<SchemaModule> {
        let ns = CString::new(ns).unwrap();
        let module =
            unsafe { ffi::ly_ctx_get_module_latest_ns(self.raw, ns.as_ptr()) };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get the (only) implemented YANG module specified by its namespace.
    pub fn get_module_implemented_ns(&self, ns: &str) -> Option<SchemaModule> {
        let ns = CString::new(ns).unwrap();
        let module = unsafe {
            ffi::ly_ctx_get_module_implemented_ns(self.raw, ns.as_ptr())
        };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get list of loaded modules.
    pub fn modules(&self) -> SchemaModules {
        SchemaModules::new(&self)
    }

    /// Returns an iterator over all data nodes from all modules in the YANG
    /// context (depth-first search algorithm).
    pub fn traverse(&self) -> impl Iterator<Item = SchemaNode> {
        self.modules()
            .flat_map(|module| module.data())
            .flat_map(|snode| snode.traverse())
    }

    /// Reset cached latest revision information of the schemas in the context.
    ///
    /// When a (sub)module is imported/included without revision, the latest
    /// revision is searched. libyang searches for the latest revision in
    /// searchdir. Then it is expected that the content of searchdirs does not
    /// change. So when it changes, it is necessary to force searching for the
    /// latest revision in case of loading another module, which what this
    /// function does.
    pub fn reset_latests(&self) {
        unsafe { ffi::ly_ctx_reset_latests(self.raw) };
    }

    /// Learn the number of internal modules of the context. Internal modules is
    /// considered one that was loaded during the context creation.
    pub fn internal_module_count(&self) -> u32 {
        unsafe { ffi::ly_ctx_internal_modules_count(self.raw) }
    }

    /// Try to find the model in the searchpaths and load it.
    ///
    /// The context itself is searched for the requested module first. If
    /// revision is not specified (the module of the latest revision is
    /// requested) and there is implemented revision of the requested module
    /// in the context, this implemented revision is returned despite there
    /// might be a newer revision. This behavior is caused by the fact that
    /// it is not possible to have multiple implemented revisions of
    /// the same module in the context.
    ///
    /// If the revision is not specified, the latest revision is loaded.
    pub fn load_module(
        &self,
        name: &str,
        revision: Option<&str>,
    ) -> Result<SchemaModule> {
        let name = CString::new(name).unwrap();
        let revision_cstr;

        let revision_ptr = match revision {
            Some(revision) => {
                revision_cstr = CString::new(revision).unwrap();
                revision_cstr.as_ptr()
            }
            None => std::ptr::null(),
        };
        let module = unsafe {
            ffi::ly_ctx_load_module(
                self.raw,
                name.as_ptr(),
                revision_ptr,
                std::ptr::null_mut(),
            )
        };
        if module.is_null() {
            return Err(Error::new(self));
        }

        Ok(SchemaModule::from_raw(self, module as *mut _))
    }

    /// Get current ID of the modules set.
    pub fn get_yanglib_id(&self) -> u16 {
        unsafe { ffi::ly_ctx_get_yanglib_id(self.raw) }
    }

    /// Get a schema node based on the given data path (JSON format).
    pub fn find_single(&self, path: &str) -> Result<SchemaNode> {
        let path = CString::new(path).unwrap();

        let rnode = unsafe {
            ffi::lys_find_path(self.raw, std::ptr::null(), path.as_ptr(), 0)
        };
        if rnode.is_null() {
            return Err(Error::new(self));
        }

        Ok(SchemaNode::from_raw(self, rnode as *mut _))
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::ly_ctx_destroy(self.raw, None) };
    }
}
