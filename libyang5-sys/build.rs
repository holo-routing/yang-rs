use std::env;
use std::path::PathBuf;

// Revision of the libyang submodule the pre-generated bindings were taken from.
#[allow(dead_code)]
const LIBYANG_REV: &str = "f302d86cd6083c2bfe16fc2122bc6d4be69ce7a2";

// Root of the WASI SDK, holding the sysroot and the CMake toolchain file used
// to cross-compile libyang and its dependencies to WebAssembly.
#[allow(dead_code)]
fn wasi_sdk() -> PathBuf {
    match env::var("WASI_SDK_PATH") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => panic!(
            "WASI_SDK_PATH must point to a WASI SDK installation when \
             targeting WebAssembly"
        ),
    }
}

// Clang's own builtin headers (stddef.h and friends) shipped with the WASI
// SDK. libclang does not locate them on its own once a foreign sysroot is in
// play, so they have to be pointed at explicitly.
#[allow(dead_code)]
fn wasi_clang_include(sdk: &std::path::Path) -> PathBuf {
    let versions = sdk.join("lib/clang");
    let entry = std::fs::read_dir(&versions)
        .unwrap_or_else(|_| {
            panic!("no clang headers under {}", versions.display())
        })
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().join("include"))
        .find(|dir| dir.join("stddef.h").exists());
    match entry {
        Some(dir) => dir,
        None => panic!("no clang builtin headers under {}", versions.display()),
    }
}

fn main() {
    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = dst.join("libyang5.rs");
    let wasm = env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32";

    // Headers of the libyang copy this crate links against, when it builds one
    // itself. Used to generate the bindings against that same revision.
    #[allow(unused_mut, unused_variables)]
    let mut include_paths: Vec<PathBuf> = vec![];

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

        if wasm {
            let sdk = wasi_sdk();
            let compat = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("wasi-compat.h");

            // yanglint and yangre are command line tools of no use here.
            cmake_config.define("ENABLE_TOOLS", "OFF");
            cmake_config.define(
                "CMAKE_TOOLCHAIN_FILE",
                sdk.join("share/cmake/wasi-sdk-p1.cmake"),
            );
            cmake_config.define("WASI_SDK_PREFIX", &sdk);

            // The toolchain file is what selects the compiler, but the cmake
            // crate probes for one of its own regardless and warns when it
            // finds nothing. Point it at the SDK to keep that quiet.
            let target = env::var("TARGET").unwrap();
            for (var, tool) in [("CC", "clang"), ("CXX", "clang++")] {
                let path = sdk.join("bin").join(tool);
                env::set_var(format!("{}_{}", var, target), &path);
                env::set_var(
                    format!("{}_{}", var, target.replace('-', "_")),
                    &path,
                );
            }

            // libyang only ever takes mutexes and rwlocks, both of which
            // wasi-libc implements for real in its single-threaded build.
            // _WASI_EMULATED_PTHREAD is needed solely to satisfy CMake's
            // FindThreads probe, which insists on pthread_create, join,
            // cancel and exit.
            cmake_config.define(
                "CMAKE_C_FLAGS",
                format!(
                    "-D_WASI_EMULATED_PTHREAD -include {}",
                    compat.display()
                ),
            );
            cmake_config
                .define("CMAKE_EXE_LINKER_FLAGS", "-lwasi-emulated-pthread");

            // pkg-config cannot find a cross-compiled PCRE2, so its location
            // has to be given explicitly.
            let pcre2_include = env::var("PCRE2_INCLUDE_DIR").expect(
                "PCRE2_INCLUDE_DIR must point to the headers of a PCRE2 \
                 built for the target when targeting WebAssembly",
            );
            let pcre2_library = env::var("PCRE2_LIBRARY").expect(
                "PCRE2_LIBRARY must point to a libpcre2-8 built for the \
                 target when targeting WebAssembly",
            );
            // The plural variables are the ones to set: FindPCRE2 takes them
            // as already resolved and skips discovery, while libyang clears
            // PCRE2_LIBRARY from the cache right before calling it.
            cmake_config.define("PCRE2_INCLUDE_DIRS", &pcre2_include);
            cmake_config.define("PCRE2_LIBRARIES", &pcre2_library);

            let pcre2_library = PathBuf::from(&pcre2_library);
            if let Some(dir) = pcre2_library.parent() {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
            println!("cargo:rustc-link-lib=static=pcre2-8");

            println!("cargo:rerun-if-env-changed=WASI_SDK_PATH");
            println!("cargo:rerun-if-env-changed=PCRE2_INCLUDE_DIR");
            println!("cargo:rerun-if-env-changed=PCRE2_LIBRARY");
            println!("cargo:rerun-if-changed=wasi-compat.h");
        }

        let cmake_dst = cmake_config.build();
        include_paths.push(cmake_dst.join("include"));

        println!("cargo:root={}", env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-link-search=native={}/lib", cmake_dst.display());
        println!(
            "cargo:rustc-link-search=native={}/lib64",
            cmake_dst.display()
        );
        if !wasm {
            if let Err(e) = pkg_config::Config::new().probe("libpcre2-8") {
                println!("cargo:warning=failed to find pcre2 library with pkg-config: {}", e);
                println!("cargo:warning=attempting to link without pkg-config");
                println!("cargo:rustc-link-lib=pcre2-8");
            }
        }
        println!("cargo:rustc-link-lib=static=yang");
        if wasm {
            let target = env::var("TARGET").unwrap();
            println!(
                "cargo:rustc-link-search=native={}",
                wasi_sdk()
                    .join("share/wasi-sysroot/lib")
                    .join(&target)
                    .display()
            );
            println!("cargo:rustc-link-lib=static=c-printscan-long-double");
            println!("cargo:rustc-link-lib=static=wasi-emulated-pthread");
        }
        println!("cargo:rerun-if-changed=libyang");
    }

    #[cfg(feature = "bindgen")]
    {
        // Add libpcre2 and libyang include paths if found in pkg-config. Only
        // meaningful for a native build against the system copies; a bundled
        // build binds the headers it just installed instead.
        if !wasm && include_paths.is_empty() {
            if let Ok(lib) = pkg_config::Config::new().probe("libpcre2-8") {
                include_paths.extend(lib.include_paths.clone());
            }
            if let Ok(lib) = pkg_config::Config::new().probe("libyang") {
                include_paths.extend(lib.include_paths.clone());
            }
        }
        // Generate Rust FFI to libyang.
        println!("cargo:rerun-if-changed=wrapper.h");
        let mut builder = bindgen::Builder::default()
            .header("wrapper.h")
            .derive_default(true)
            .default_enum_style(bindgen::EnumVariation::ModuleConsts)
            // Bind libyang's own declarations, plus the free() its callers
            // need to release the strings it hands out.
            .allowlist_file(".*/libyang/.*\\.h")
            .allowlist_function("free")
            // Plugin directories baked in by libyang's own build. They are of
            // no use from Rust, and keeping them would tie the pre-generated
            // bindings to the machine they were generated on.
            .blocklist_item("LYPLG_TYPE_DIR")
            .blocklist_item("LYPLG_EXT_DIR");
        for path in &include_paths {
            builder = builder.clang_arg(format!("-I{}", path.display()));
        }
        if wasm {
            // Bind against wasi-libc rather than the host libc, so that the
            // layout of every struct matches the 32-bit target.
            let sdk = wasi_sdk();
            let sysroot = sdk.join("share/wasi-sysroot");
            builder = builder
                .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
                .clang_arg(format!("--sysroot={}", sysroot.display()))
                .clang_arg(format!("-I{}", wasi_clang_include(&sdk).display()))
                .clang_arg("-D_WASI_EMULATED_PTHREAD")
                // Symbols default to hidden visibility on WebAssembly, and
                // bindgen skips hidden functions. Without this only the few
                // declarations libyang marks LIBYANG_API_DEF come through.
                .clang_arg("-fvisibility=default");
        }
        let bindings = builder
            .generate()
            .expect("Unable to generate libyang5 bindings");
        bindings
            .write_to_file(out_file)
            .expect("Couldn't write libyang5 bindings!");
    }
    #[cfg(not(feature = "bindgen"))]
    {
        // The bindings encode the layout of libyang's structs, which differs
        // between the 64-bit hosts and 32-bit WebAssembly.
        let bindings = match wasm {
            true => format!("libyang5-{}-wasm32.rs", LIBYANG_REV),
            false => format!("libyang5-{}.rs", LIBYANG_REV),
        };
        let mut pregen_bindings = PathBuf::new();
        pregen_bindings.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        pregen_bindings.push("pre-generated-bindings");
        pregen_bindings.push(bindings);

        std::fs::copy(&pregen_bindings, &out_file)
            .expect("Unable to copy pre-generated libyang5 bindings");
    }

    #[cfg(not(feature = "bundled"))]
    {
        // pkg-config is of no use when cross-compiling; the search path for a
        // libyang built for the target has to be supplied by the caller.
        if wasm {
            println!("cargo:rustc-link-lib=static=yang");
            println!("cargo:rustc-link-lib=static=pcre2-8");
        } else if let Err(e) = pkg_config::Config::new().probe("libyang") {
            println!(
                "cargo:warning=failed to find yang library with pkg-config: {}",
                e
            );
            println!("cargo:warning=attempting to link without pkg-config");
            println!("cargo:rustc-link-lib=yang");
        }
    }
}
