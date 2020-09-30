use yang2::context::{Context, ContextFlags};
use yang2::schema::{SchemaOutputFormat, SchemaPrinterFlags};

static SEARCH_DIR: &str = "./assets/yang/";
static MODULE_NAME: &str = "ietf-routing";

fn main() -> std::io::Result<()> {
    // Initialize context.
    let ctx = Context::new(SEARCH_DIR, ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Load test module.
    let module = ctx
        .load_module(MODULE_NAME, None)
        .expect("Failed to load module");

    // Print test module.
    module
        .print_file(
            std::io::stdout(),
            SchemaOutputFormat::YIN,
            SchemaPrinterFlags::empty(),
        )
        .expect("Failed to print module");

    Ok(())
}
