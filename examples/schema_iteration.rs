use yang3::context::{Context, ContextFlags};
use yang3::schema::SchemaPathFormat;

static SEARCH_DIR: &str = "./assets/yang/";
static MODULE_NAME: &str = "ietf-isis";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");

    // Load test module.
    ctx.load_module(MODULE_NAME, None, &[])
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
    let module = ctx
        .get_module_latest(MODULE_NAME)
        .expect("Failed to find loaded module");
    for snode in module.rpcs() {
        println!("  {}", snode.path(SchemaPathFormat::DATA));
    }

    println!("Notifications:");
    for snode in module.notifications() {
        println!("  {}", snode.path(SchemaPathFormat::DATA));
    }

    Ok(())
}
