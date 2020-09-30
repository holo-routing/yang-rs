use std::fs::File;
use yang2::context::{Context, ContextFlags};
use yang2::data::{
    Data, DataFormat, DataParserFlags, DataTree, DataValidationFlags,
};

static SEARCH_DIR: &str = "./assets/yang/";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let ctx = Context::new(SEARCH_DIR, ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Load YANG modules.
    for module_name in &["ietf-interfaces", "iana-if-type", "ietf-isis"] {
        ctx.load_module(module_name, None)
            .expect("Failed to load module");
    }

    // Parse data tree in the JSON format.
    let dtree = DataTree::parse_file(
        &ctx,
        File::open("./assets/data/isis.json")?,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data tree");

    // Iterate over all nodes of the data tree.
    println!("Iterating over all data nodes...");
    for dnode in dtree.traverse() {
        println!("  {}: {:?}", dnode.path().unwrap(), dnode.value());
    }

    // Iterate over all interfaces present in the data tree.
    println!("Iterating over interfaces only...");
    let dnodes = dtree
        .find("/ietf-interfaces:interfaces/interface")
        .expect("Failed to find interfaces");
    for dnode in dnodes {
        println!("  {}: {:?}", dnode.path().unwrap(), dnode.value());
    }

    Ok(())
}
