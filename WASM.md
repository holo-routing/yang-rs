# WebAssembly

yang-rs builds and runs on `wasm32-wasip1`, with libyang and PCRE2 cross compiled by the [WASI SDK]. There is no system libyang to link against there, so the **bundled** feature is enabled automatically and does not need to be passed. Three variables tell the build where to find the pieces:

```sh
export WASI_SDK_PATH=/path/to/wasi-sdk
export PCRE2_INCLUDE_DIR=/path/to/pcre2/include
export PCRE2_LIBRARY=/path/to/pcre2/lib/libpcre2-8.a
cargo build --target wasm32-wasip1
```

PCRE2 has to be cross compiled beforehand, with the JIT turned off as it has no WebAssembly support:

```sh
cmake -S pcre2 -B pcre2-build -DCMAKE_TOOLCHAIN_FILE=$WASI_SDK_PATH/share/cmake/wasi-sdk-p1.cmake -DWASI_SDK_PREFIX=$WASI_SDK_PATH -DCMAKE_INSTALL_PREFIX=$PWD/prefix -DBUILD_SHARED_LIBS=OFF -DPCRE2_SUPPORT_JIT=OFF -DPCRE2_BUILD_TESTS=OFF
cmake --build pcre2-build --target install
```

To run the test suite, point cargo at a WebAssembly runtime and give it access to the package directory, which the tests read their assets from:

```sh
CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime run --dir ." cargo test --target wasm32-wasip1
```

[WASI SDK]: https://github.com/WebAssembly/wasi-sdk
