use std::env;
use std::path::PathBuf;

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = dst.join("libyang2.rs");

    #[cfg(feature = "use_bindgen")]
    {
        // Generate Rust FFI to libfrr.
        println!("cargo:rustc-link-lib=yang");
        println!("cargo:rerun-if-changed=wrapper.h");
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg("-DLY_ENABLED_LYD_PRIV")
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
            .push("libyang2-de8d5cc7b9bf4fcce1007c8ff3d04d6000cdd081.rs");

        std::fs::copy(&pregen_bindings, &out_file)
            .expect("Unable to copy pre-generated libyang2 bindings");
    }
}
