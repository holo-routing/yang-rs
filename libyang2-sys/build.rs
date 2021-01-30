use std::env;
use std::path::PathBuf;

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = dst.join("libyang2.rs");

    println!("cargo:rustc-link-lib=yang");

    #[cfg(feature = "use_bindgen")]
    {
        // Generate Rust FFI to libfrr.
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
            .push("libyang2-59618972be7982dd5a22ea55fc44b04b8b80734c.rs");

        std::fs::copy(&pregen_bindings, &out_file)
            .expect("Unable to copy pre-generated libyang2 bindings");
    }
}
