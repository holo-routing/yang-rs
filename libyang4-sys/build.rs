use std::env;
use std::path::PathBuf;

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = dst.join("libyang4.rs");

    #[cfg(feature = "bindgen")]
    {
        let mut include_paths = vec![];
        // Add libpcre2 include paths if found in pkg-config
        if let Ok(lib) = pkg_config::Config::new().probe("libpcre2-8") {
            include_paths = lib.include_paths.clone();
        }
        // Add libyang include paths if found in pkg-config
        if let Ok(lib) = pkg_config::Config::new().probe("libyang") {
            include_paths.extend(lib.include_paths.clone());
        }
        // Generate Rust FFI to libyang.
        println!("cargo:rerun-if-changed=wrapper.h");
        let mut builder = bindgen::Builder::default()
            .header("wrapper.h")
            .derive_default(true)
            .default_enum_style(bindgen::EnumVariation::ModuleConsts);
        for path in &include_paths {
            builder = builder.clang_arg(format!("-I{}", path.display()));
        }
        let bindings = builder
            .generate()
            .expect("Unable to generate libyang4 bindings");
        bindings
            .write_to_file(out_file)
            .expect("Couldn't write libyang4 bindings!");
    }
    #[cfg(not(feature = "bindgen"))]
    {
        let mut pregen_bindings = PathBuf::new();
        pregen_bindings.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        pregen_bindings.push("pre-generated-bindings");
        pregen_bindings
            .push("libyang4-3d07c3a71534a580c3960907da17568eff7e5c64.rs");

        std::fs::copy(&pregen_bindings, &out_file)
            .expect("Unable to copy pre-generated libyang4 bindings");
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
        // Run cmake configure and build pcre2 and libyang
        let mut pcre2_config = cmake::Config::new("pcre2");
        pcre2_config.define("BUILD_SHARED_LIBS", "OFF");
        pcre2_config.define("PCRE2_STATIC_PIC", "ON");
        pcre2_config.define("PCRE2_SUPPORT_JIT", "OFF");
        pcre2_config.define("PCRE2_BUILD_TESTS", "OFF");
        pcre2_config.define("PCRE2_BUILD_PCRE2GREP", "OFF");
        env::set_var("DEP_PCRE2_ROOT", pcre2_config.build());
        let mut cmake_config = cmake::Config::new("libyang");
        cmake_config.register_dep("PCRE2");
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
        println!("cargo:rustc-link-lib=static=yang");
        println!("cargo:rerun-if-changed=libyang");
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
