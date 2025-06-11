use std::env;
use std::path::PathBuf;

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = dst.join("libyang3.rs");

    #[cfg(feature = "bindgen")]
    #[cfg(not(feature = "bundled"))]
    {
        // Generate Rust FFI to libyang.
        println!("cargo:rerun-if-changed=wrapper.h");
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .derive_default(true)
            .default_enum_style(bindgen::EnumVariation::ModuleConsts)
            .generate()
            .expect("Unable to generate libyang3 bindings");
        bindings
            .write_to_file(out_file)
            .expect("Couldn't write libyang3 bindings!");
    }
    #[cfg(not(feature = "bindgen"))]
    {
        let mut pregen_bindings = PathBuf::new();
        pregen_bindings.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        pregen_bindings.push("pre-generated-bindings");
        pregen_bindings
            .push("libyang3-f313632a8ff45d7bba4be29ea9dc83ab5b533807.rs");

        std::fs::copy(&pregen_bindings, &out_file)
            .expect("Unable to copy pre-generated libyang3 bindings");
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
        // Run cmake configure and build libyang
        let mut cmake_config = cmake::Config::new("libyang");
        cmake_config.define("BUILD_SHARED_LIBS", "OFF"); // Force static linking
        cmake_config.define("ENABLE_TESTS", "OFF");
        cmake_config.define("ENABLE_VALGRIND_TESTS", "OFF");
        cmake_config.define("ENABLE_BUILD_TESTS", "OFF");
        cmake_config.define("CMAKE_BUILD_TYPE", "Release");
        cmake_config.define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
        let cmake_dst = cmake_config.build();
        println!("cargo:root={}", env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-link-search=native={}/lib", cmake_dst.display());
        println!(
            "cargo:rustc-link-search=native={}/lib64",
            cmake_dst.display()
        );
        if let Err(e) = pkg_config::Config::new().probe("libpcre2-8") {
            println!("cargo:warning=failed to find pcre2 library with pkg-config: {}", e);
            println!("cargo:warning=attempting to link without pkg-config");
            println!("cargo:rustc-link-lib=pcre2-8");
        }
        println!("cargo:rustc-link-lib=static=yang");
        println!("cargo:rerun-if-changed=libyang");

        #[cfg(feature = "bindgen")]
        {
            // Use the newly compiled libyang code to generate Rust FFI to libyang
            println!("cargo:rerun-if-changed=wrapper.h");
            let bindings = bindgen::Builder::default()
                .header("wrapper.h")
                .clang_arg(format!("-I{}/include", cmake_dst.display()))
                .derive_default(true)
                .default_enum_style(bindgen::EnumVariation::ModuleConsts)
                .generate()
                .expect("Unable to generate libyang3 bindings");
            bindings
                .write_to_file(out_file)
                .expect("Couldn't write libyang3 bindings!");
        }
    }
    #[cfg(not(feature = "bundled"))]
    {
        if let Err(e) = pkg_config::Config::new().probe("libyang") {
            println!(
                "cargo:warning=failed to find yang library with pkg-config: {}",
                e
            );
            println!("cargo:warning=attempting to link without pkg-config");
            println!("cargo:rustc-link-lib=yang");
        }
    }
}
