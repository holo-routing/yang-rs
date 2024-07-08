use std::fs::File;
use std::sync::Arc;
use yang3::context::{Context, ContextFlags};
use yang3::data::{
    Data, DataFormat, DataParserFlags, DataTree, DataValidationFlags,
};

static SEARCH_DIR: &str = "./assets/yang/";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");

    // Load YANG modules.
    for module_name in &["ietf-interfaces", "iana-if-type", "ietf-isis"] {
        ctx.load_module(module_name, None, &[])
            .expect("Failed to load module");
    }
    let ctx = Arc::new(ctx);

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
        println!("  {}: {:?}", dnode.path(), dnode.value());
    }

    // Iterate over all interfaces present in the data tree.
    println!("Iterating over interfaces only...");
    for dnode in dtree
        .find_xpath("/ietf-interfaces:interfaces/interface")
        .expect("Failed to find interfaces")
    {
        println!("  {}: {:?}", dnode.path(), dnode.value());
    }

    Ok(())
}
