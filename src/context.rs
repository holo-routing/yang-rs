//
// Copyright (c) The yang2-rs Core Contributors
//
// See LICENSE for license details.
//

//! YANG context.

use bitflags::bitflags;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::slice;
use std::sync::Once;

use crate::error::{Error, Result};
use crate::iter::{SchemaModules, Set};
use crate::schema::{SchemaModule, SchemaNode};
use crate::utils::*;
use libyang2_sys as ffi;

/// Context of the YANG schemas.
///
/// [Official C documentation]
///
/// [Official C documentation]: https://netopeer.liberouter.org/doc/libyang/libyang2/html/howto_context.html
#[derive(Debug, PartialEq)]
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

/// Embedded module key containing the module/submodule name and optional
/// revision.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct EmbeddedModuleKey {
    mod_name: &'static str,
    mod_rev: Option<&'static str>,
    submod_name: Option<&'static str>,
    submod_rev: Option<&'static str>,
}

/// A hashmap containing embedded YANG modules.
pub type EmbeddedModules = HashMap<EmbeddedModuleKey, &'static str>;

// ===== impl Context =====

impl Context {
    /// Create libyang context.
    ///
    /// Context is used to hold all information about schemas. Usually, the
    /// application is supposed to work with a single context in which
    /// libyang is holding all schemas (and other internal information)
    /// according to which the data trees will be processed and validated.
    pub fn new(options: ContextFlags) -> Result<Context> {
        static INIT: Once = Once::new();
        let mut context = std::ptr::null_mut();
        let ctx_ptr = &mut context;

        // Initialization routine that is called only once when the first YANG
        // context is created.
        INIT.call_once(|| {
            // Disable automatic logging to stderr in order to give users more
            // control over the handling of errors.
            unsafe { ffi::ly_log_options(ffi::LY_LOSTORE_LAST) };
        });

        let ret =
            unsafe { ffi::ly_ctx_new(std::ptr::null(), options.bits, ctx_ptr) };
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
    pub fn set_searchdir<P: AsRef<Path>>(
        &mut self,
        search_dir: P,
    ) -> Result<()> {
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
    pub fn unset_searchdir<P: AsRef<Path>>(
        &mut self,
        search_dir: P,
    ) -> Result<()> {
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
    pub fn unset_searchdirs(&mut self) -> Result<()> {
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
    pub fn unset_searchdir_last(&mut self, count: u32) -> Result<()> {
        let ret = unsafe { ffi::ly_ctx_unset_searchdir_last(self.raw, count) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Set hash map containing embedded YANG modules, which are loaded on
    /// demand.
    pub fn set_embedded_modules(&mut self, modules: &EmbeddedModules) {
        unsafe {
            ffi::ly_ctx_set_module_imp_clb(
                self.raw,
                Some(ly_module_import_cb),
                modules as *const _ as *mut c_void,
            )
        };
    }

    /// Remove all embedded modules from the libyang context.
    pub fn unset_embedded_modules(&mut self) {
        unsafe {
            ffi::ly_ctx_set_module_imp_clb(self.raw, None, std::ptr::null_mut())
        };
    }

    /// Get the currently set context's options.
    pub fn get_options(&self) -> ContextFlags {
        let options = unsafe { ffi::ly_ctx_get_options(self.raw) };
        ContextFlags::from_bits_truncate(options)
    }

    /// Set some of the context's options.
    pub fn set_options(&mut self, options: ContextFlags) -> Result<()> {
        let ret = unsafe { ffi::ly_ctx_set_options(self.raw, options.bits) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Unset some of the context's options.
    pub fn unset_options(&mut self, options: ContextFlags) -> Result<()> {
        let ret = unsafe { ffi::ly_ctx_unset_options(self.raw, options.bits) };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        Ok(())
    }

    /// Get current ID of the modules set.
    pub fn get_module_set_id(&self) -> u16 {
        unsafe { ffi::ly_ctx_get_change_count(self.raw) }
    }

    /// Get YANG module of the given name and revision.
    ///
    /// If the revision is not specified, the schema with no revision is
    /// returned (if it is present in the context).
    pub fn get_module(
        &self,
        name: &str,
        revision: Option<&str>,
    ) -> Option<SchemaModule<'_>> {
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
    pub fn get_module_latest(&self, name: &str) -> Option<SchemaModule<'_>> {
        let name = CString::new(name).unwrap();
        let module =
            unsafe { ffi::ly_ctx_get_module_latest(self.raw, name.as_ptr()) };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get the (only) implemented YANG module specified by its name.
    pub fn get_module_implemented(
        &self,
        name: &str,
    ) -> Option<SchemaModule<'_>> {
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
    ) -> Option<SchemaModule<'_>> {
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
    pub fn get_module_latest_ns(&self, ns: &str) -> Option<SchemaModule<'_>> {
        let ns = CString::new(ns).unwrap();
        let module =
            unsafe { ffi::ly_ctx_get_module_latest_ns(self.raw, ns.as_ptr()) };
        if module.is_null() {
            return None;
        }

        Some(SchemaModule::from_raw(self, module))
    }

    /// Get the (only) implemented YANG module specified by its namespace.
    pub fn get_module_implemented_ns(
        &self,
        ns: &str,
    ) -> Option<SchemaModule<'_>> {
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
    ///
    /// Internal modules (loaded during the context creation) can be skipped by
    /// setting "skip_internal" to true.
    pub fn modules(&self, skip_internal: bool) -> SchemaModules<'_> {
        SchemaModules::new(self, skip_internal)
    }

    /// Returns an iterator over all data nodes from all modules in the YANG
    /// context (depth-first search algorithm).
    pub fn traverse(&self) -> impl Iterator<Item = SchemaNode<'_>> {
        self.modules(false).flat_map(|module| module.traverse())
    }

    /// Reset cached latest revision information of the schemas in the context.
    ///
    /// When a (sub)module is imported/included without revision, the latest
    /// revision is searched. libyang searches for the latest revision in
    /// searchdir. Then it is expected that the content of searchdirs does not
    /// change. So when it changes, it is necessary to force searching for the
    /// latest revision in case of loading another module, which what this
    /// function does.
    pub fn reset_latests(&mut self) {
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
    ///
    /// The `features` parameter specifies the module features that should be
    /// enabled. If let empty, no features are enabled. The feature string '*'
    /// enables all module features.
    pub fn load_module(
        &mut self,
        name: &str,
        revision: Option<&str>,
        features: &[&str],
    ) -> Result<SchemaModule<'_>> {
        let name = CString::new(name).unwrap();
        let revision_cstr;
        let features_cstr;
        let mut features_ptr;

        // Prepare revision string.
        let revision_ptr = match revision {
            Some(revision) => {
                revision_cstr = CString::new(revision).unwrap();
                revision_cstr.as_ptr()
            }
            None => std::ptr::null(),
        };

        // Prepare features array.
        features_cstr = features
            .iter()
            .map(|feature| CString::new(*feature).unwrap())
            .collect::<Vec<_>>();
        features_ptr = features_cstr
            .iter()
            .map(|feature| feature.as_ptr())
            .collect::<Vec<_>>();
        features_ptr.push(std::ptr::null());

        let module = unsafe {
            ffi::ly_ctx_load_module(
                self.raw,
                name.as_ptr(),
                revision_ptr,
                features_ptr.as_mut_ptr(),
            )
        };
        if module.is_null() {
            return Err(Error::new(self));
        }

        Ok(SchemaModule::from_raw(self, module as *mut _))
    }

    /// Evaluate an xpath expression on schema nodes.
    pub fn find_xpath(&self, path: &str) -> Result<Set<'_, SchemaNode<'_>>> {
        let path = CString::new(path).unwrap();
        let mut set = std::ptr::null_mut();
        let set_ptr = &mut set;
        let options = 0u32;

        let ret = unsafe {
            ffi::lys_find_xpath(
                self.raw,
                std::ptr::null(),
                path.as_ptr(),
                options,
                set_ptr,
            )
        };
        if ret != ffi::LY_ERR::LY_SUCCESS {
            return Err(Error::new(self));
        }

        let rnodes_count = unsafe { (*set).count } as usize;
        let slice = if rnodes_count == 0 {
            &[]
        } else {
            let rnodes = unsafe { (*set).__bindgen_anon_1.snodes };
            unsafe { slice::from_raw_parts(rnodes, rnodes_count) }
        };

        Ok(Set::new(self, slice))
    }

    /// Get a schema node based on the given data path (JSON format).
    pub fn find_path(&self, path: &str) -> Result<SchemaNode<'_>> {
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

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::ly_ctx_destroy(self.raw) };
    }
}

// ===== impl EmbeddedModuleKey =====

impl EmbeddedModuleKey {
    pub fn new(
        mod_name: &'static str,
        mod_rev: Option<&'static str>,
        submod_name: Option<&'static str>,
        submod_rev: Option<&'static str>,
    ) -> EmbeddedModuleKey {
        EmbeddedModuleKey {
            mod_name,
            mod_rev,
            submod_name,
            submod_rev,
        }
    }
}

// ===== helper functions =====

fn find_embedded_module<'a>(
    modules: &'a EmbeddedModules,
    mod_name: &'a str,
    mod_rev: Option<&'a str>,
    submod_name: Option<&'a str>,
    submod_rev: Option<&'a str>,
) -> Option<(&'a EmbeddedModuleKey, &'a &'a str)> {
    modules.iter().find(|(key, _)| {
        *key.mod_name == *mod_name
            && (mod_rev.is_none() || key.mod_rev == mod_rev)
            && match submod_name {
                Some(submod_name) => {
                    key.submod_name == Some(submod_name)
                        && (submod_rev.is_none()
                            || key.submod_rev == submod_rev)
                }
                None => key.submod_name.is_none(),
            }
    })
}

unsafe extern "C" fn ly_module_import_cb(
    mod_name: *const c_char,
    mod_rev: *const c_char,
    submod_name: *const c_char,
    submod_rev: *const c_char,
    user_data: *mut c_void,
    format: *mut ffi::LYS_INFORMAT::Type,
    module_data: *mut *const c_char,
    _free_module_data: *mut ffi::ly_module_imp_data_free_clb,
) -> ffi::LY_ERR::Type {
    let modules = &*(user_data as *const EmbeddedModules);
    let mod_name = char_ptr_to_str(mod_name);
    let mod_rev = char_ptr_to_opt_str(mod_rev);
    let submod_name = char_ptr_to_opt_str(submod_name);
    let submod_rev = char_ptr_to_opt_str(submod_rev);

    if let Some((_emod_key, emod_data)) = find_embedded_module(
        modules,
        mod_name,
        mod_rev,
        submod_name,
        submod_rev,
    ) {
        let data = CString::new(*emod_data).unwrap();

        *format = ffi::LYS_INFORMAT::LYS_IN_YANG;
        *module_data = data.as_ptr();
        std::mem::forget(data);
        return ffi::LY_ERR::LY_SUCCESS;
    }

    ffi::LY_ERR::LY_ENOTFOUND
}
