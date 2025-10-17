use yang3::context::{Context, ContextFlags};

static SEARCH_DIR: &str = "./assets/yang/";
static MODULE_NAME: &str = "ietf-isis";

fn main() -> std::io::Result<()> {
    // Initialize context
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Set search directory
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");

    // Load the module
    let module = ctx
        .load_module(MODULE_NAME, None, &[])
        .expect("Failed to load module");

    // Get imports for the loaded module
    let imports: Vec<_> = module.imports().collect();

    println!("Module '{}' imports:\n", module.name());

    // Check methods
    for import in &imports {
        println!("  Import: {}", import.name());
        println!("    Prefix: {}", import.prefix());

        if let Some(description) = import.description() {
            println!("    Description: {}", description);
        }

        if let Some(reference) = import.reference() {
            let reference_oneline =
                reference.replace('\n', " ").replace('\r', " ");
            println!("    Reference: {}", reference_oneline);
        }

        // Check module() method
        let imported_module = import.module();
        println!("      Name: {}", imported_module.name());
        println!("      Namespace: {}", imported_module.namespace());

        if let Some(filepath) = imported_module.filepath() {
            println!("      File path: {}", filepath);
        }

        if let Some(revision) = imported_module.revision() {
            println!("      Revision: {}", revision);
        }
        println!()
    }

    Ok(())
}
