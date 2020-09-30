# yang2-rs

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/yang2.svg
[crates-url]: https://crates.io/crates/yang2
[docs-badge]: https://docs.rs/yang2/badge.svg
[docs-url]: https://docs.rs/yang2
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/rwestphal/yang2-rs/blob/master/LICENSE
[actions-badge]: https://github.com/rwestphal/yang2-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/rwestphal/yang2-rs/actions?query=workflow%3ACI+branch%3Amaster

Rust bindings for the [libyang2] library.

For raw FFI bindings for libyang2, see [libyang2-sys].

[libyang2]: https://github.com/CESNET/libyang/tree/libyang2
[libyang2-sys]: https://github.com/rwestphal/yang2-rs/tree/master/libyang2-sys

#### Cargo.toml

```toml
[dependencies]
yang2 = "0.1"
```
## Design Goals
* Provide high-level bindings for libyang2 using idiomatic Rust
* Leverage Rust's ownership system to detect API misuse problems at compile time
* Automatic resource management
* Zero-cost abstractions

## Example

A basic example that parses and validates JSON instance data, and then converts
it to the XML format:
```rust,no_run
use std::fs::File;
use yang2::context::{Context, ContextFlags};
use yang2::data::{
    Data, DataFormat, DataParserFlags, DataPrinterFlags, DataTree,
    DataValidationFlags,
};

static SEARCH_DIR: &str = "./assets/yang/";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let ctx = Context::new(SEARCH_DIR, ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Load YANG modules.
    for module_name in &["ietf-interfaces", "iana-if-type"] {
        ctx.load_module(module_name, None)
            .expect("Failed to load module");
    }

    // Parse and validate data tree in the JSON format.
    let dtree = DataTree::parse_file(
        &ctx,
        File::open("./assets/data/interfaces.json")?,
        DataFormat::JSON,
        DataParserFlags::empty(),
        DataValidationFlags::empty(),
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

More examples can be found [here][examples].

[examples]: https://github.com/rwestphal/yang2-rs/tree/master/examples

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/rwestphal/yang2-rs/blob/master/LICENSE

### Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/rwestphal/yang2-rs.
