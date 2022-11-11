use std::env;
use std::path::PathBuf;

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = dst.join("libyang2.rs");

    #[cfg(feature = "use_bindgen")]
    {
        // Generate Rust FFI to libfrr.
        println!("cargo:rerun-if-changed=wrapper.h");
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .derive_default(true)
            .default_enum_style(bindgen::EnumVariation::ModuleConsts)
            .generate()
            .expect("Unable to generate libyang2 bindings");
        bindings
            .write_to_file(out_file)
            .expect("Couldn't write libyang2 bindings!");
    }
    #[cfg(not(feature = "use_bindgen"))]
    {
        let mut pregen_bindings = PathBuf::new();
        pregen_bindings.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        pregen_bindings.push("pre-generated-bindings");
        pregen_bindings
            .push("libyang2-c7e6136c3226f0c6a95d598ff3ab69c8e89b9a40.rs");

        std::fs::copy(&pregen_bindings, &out_file)
            .expect("Unable to copy pre-generated libyang2 bindings");
    }

    #[cfg(feature = "bundled")]
    {
        use std::path::Path;
        use std::process::Command;

        // Initialize the libyang submodule if necessary.
        if !Path::new("libyang/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status();
        }

        // Run cmake.
        let cmake_dst = cmake::build("libyang");

        // Build libyang2.
        let mut build = cc::Build::new();
        build
            .include(format!("{}/build/compat", cmake_dst.display()))
            .include(format!("{}/build/src", cmake_dst.display()))
            .include("libyang/src")
            .include("libyang/src/plugins_exts")
            .file("libyang/compat/compat.c")
            .file("libyang/src/common.c")
            .file("libyang/src/context.c")
            .file("libyang/src/diff.c")
            .file("libyang/src/hash_table.c")
            .file("libyang/src/in.c")
            .file("libyang/src/json.c")
            .file("libyang/src/log.c")
            .file("libyang/src/lyb.c")
            .file("libyang/src/out.c")
            .file("libyang/src/parser_common.c")
            .file("libyang/src/parser_json.c")
            .file("libyang/src/parser_lyb.c")
            .file("libyang/src/parser_xml.c")
            .file("libyang/src/parser_yang.c")
            .file("libyang/src/parser_yin.c")
            .file("libyang/src/path.c")
            .file("libyang/src/plugins.c")
            .file("libyang/src/plugins_exts.c")
            .file("libyang/src/plugins_exts/metadata.c")
            .file("libyang/src/plugins_exts/nacm.c")
            .file("libyang/src/plugins_exts/schema_mount.c")
            .file("libyang/src/plugins_exts/structure.c")
            .file("libyang/src/plugins_exts/yangdata.c")
            .file("libyang/src/plugins_types.c")
            .file("libyang/src/plugins_types/binary.c")
            .file("libyang/src/plugins_types/bits.c")
            .file("libyang/src/plugins_types/boolean.c")
            .file("libyang/src/plugins_types/date_and_time.c")
            .file("libyang/src/plugins_types/decimal64.c")
            .file("libyang/src/plugins_types/empty.c")
            .file("libyang/src/plugins_types/enumeration.c")
            .file("libyang/src/plugins_types/identityref.c")
            .file("libyang/src/plugins_types/instanceid.c")
            .file("libyang/src/plugins_types/instanceid_keys.c")
            .file("libyang/src/plugins_types/integer.c")
            .file("libyang/src/plugins_types/ipv4_address.c")
            .file("libyang/src/plugins_types/ipv4_address_no_zone.c")
            .file("libyang/src/plugins_types/ipv4_prefix.c")
            .file("libyang/src/plugins_types/ipv6_address.c")
            .file("libyang/src/plugins_types/ipv6_address_no_zone.c")
            .file("libyang/src/plugins_types/ipv6_prefix.c")
            .file("libyang/src/plugins_types/leafref.c")
            .file("libyang/src/plugins_types/node_instanceid.c")
            .file("libyang/src/plugins_types/string.c")
            .file("libyang/src/plugins_types/union.c")
            .file("libyang/src/plugins_types/xpath1.0.c")
            .file("libyang/src/printer_data.c")
            .file("libyang/src/printer_json.c")
            .file("libyang/src/printer_lyb.c")
            .file("libyang/src/printer_schema.c")
            .file("libyang/src/printer_tree.c")
            .file("libyang/src/printer_xml.c")
            .file("libyang/src/printer_yang.c")
            .file("libyang/src/printer_yin.c")
            .file("libyang/src/schema_compile_amend.c")
            .file("libyang/src/schema_compile.c")
            .file("libyang/src/schema_compile_node.c")
            .file("libyang/src/schema_features.c")
            .file("libyang/src/set.c")
            .file("libyang/src/tree_data.c")
            .file("libyang/src/tree_data_common.c")
            .file("libyang/src/tree_data_free.c")
            .file("libyang/src/tree_data_hash.c")
            .file("libyang/src/tree_data_new.c")
            .file("libyang/src/tree_schema.c")
            .file("libyang/src/tree_schema_common.c")
            .file("libyang/src/tree_schema_free.c")
            .file("libyang/src/validation.c")
            .file("libyang/src/xml.c")
            .file("libyang/src/xpath.c")
            .warnings(false);

        build.compile("yang2");
        println!("cargo:root={}", env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-link-lib=pcre2-8");
    }
    #[cfg(not(feature = "bundled"))]
    println!("cargo:rustc-link-lib=yang");
}
