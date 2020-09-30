use std::env;
use std::path::PathBuf;

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());

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
        .write_to_file(dst.join("libyang2.rs"))
        .expect("Couldn't write libyang2 bindings!");
}
