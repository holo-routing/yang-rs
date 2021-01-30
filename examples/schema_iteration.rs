use yang2::context::{Context, ContextFlags};
use yang2::schema::SchemaPathFormat;

static SEARCH_DIR: &str = "./assets/yang/";
static MODULE_NAME: &str = "ietf-isis";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let ctx = Context::new(SEARCH_DIR, ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Load test module.
    let module = ctx
        .load_module(MODULE_NAME, None)
        .expect("Failed to load module");

    // Iterate over all schema nodes that belong to the test module and print
    // their full paths.
    println!("Data (DFS iteration):");
    for snode in ctx
        .traverse()
        .filter(|snode| snode.module().name() == MODULE_NAME)
    {
        println!("  {}", snode.path(SchemaPathFormat::DATA));
    }

    println!("RPCs:");
    for snode in module.rpcs() {
        println!("  {}", snode.path(SchemaPathFormat::DATA));
    }

    println!("Notifications:");
    for snode in module.notifications() {
        println!("  {}", snode.path(SchemaPathFormat::DATA));
    }

    Ok(())
}
