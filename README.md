# yang-rs

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![codecov][codecov-badge]][codecov-url]

[crates-badge]: https://img.shields.io/crates/v/yang3.svg
[crates-url]: https://crates.io/crates/yang3
[docs-badge]: https://docs.rs/yang3/badge.svg
[docs-url]: https://docs.rs/yang3
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/holo-routing/yang-rs/blob/master/LICENSE
[actions-badge]: https://github.com/holo-routing/yang-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/holo-routing/yang-rs/actions?query=workflow%3ACI+branch%3Amaster
[codecov-badge]: https://codecov.io/gh/holo-routing/yang-rs/branch/master/graph/badge.svg?token=1KE3JMHG0H
[codecov-url]: https://codecov.io/gh/holo-routing/yang-rs

Rust bindings for the [libyang] library.

For raw FFI bindings for libyang, see [libyang3-sys].

[libyang]: https://github.com/CESNET/libyang/
[libyang3-sys]: ./libyang3-sys

#### Cargo.toml

```toml
[dependencies]
yang3 = "0.7"
```
## Design Goals
* Provide high-level bindings for libyang using idiomatic Rust
* Leverage Rust's ownership system to detect API misuse problems at compile time
* Automatic resource management
* Zero-cost abstractions

## Feature flags
By default, yang-rs uses pre-generated FFI bindings and uses dynamic linking to load libyang. The following feature flags, however, can be used to change that behavior:
* **bundled**: instructs cargo to download and build libyang from the sources. The resulting objects are grouped into a static archive linked to this crate. This feature can be used when having a libyang dynamic link dependency isn't desirable.
  * Additional build requirements: *cc 1.0*, *cmake 0.1*, a C compiler and CMake.
* **use_bindgen**: generate new C FFI bindings dynamically instead of using the pre-generated ones. Useful when updating this crate to use newer libyang versions.
  * Additional build requirements: *bindgen 0.68.0*

## Example

A basic example that parses and validates JSON instance data, and then converts
it to the XML format:
```rust,no_run
use std::fs::File;
use yang3::context::{Context, ContextFlags};
use yang3::data::{
    Data, DataFormat, DataParserFlags, DataPrinterFlags, DataTree,
    DataValidationFlags,
};

static SEARCH_DIR: &str = "./assets/yang/";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");

    // Load YANG modules.
    for module_name in &["ietf-interfaces", "iana-if-type"] {
        ctx.load_module(module_name, None, &[])
            .expect("Failed to load module");
    }

    // Parse and validate data tree in the JSON format.
    let dtree = DataTree::parse_file(
        &ctx,
        File::open("./assets/data/interfaces.json")?,
        DataFormat::JSON,
        DataParserFlags::empty(),
        DataValidationFlags::NO_STATE,
    )
    .expect("Failed to parse data tree");

    // Print data tree in the XML format.
    dtree
        .print_file(
            std::io::stdout(),
            DataFormat::XML,
            DataPrinterFlags::WD_ALL | DataPrinterFlags::WITH_SIBLINGS,
        )
        .expect("Failed to print data tree");

    Ok(())
}
```

Note the `NO_STATE` flag passed to `parse_file` since the example json file does not contain state data.
More examples can be found [here][examples].

[examples]: https://github.com/holo-routing/yang-rs/tree/master/examples

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/holo-routing/yang-rs/blob/master/LICENSE

### Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/holo-routing/yang-rs.
