use std::fs::File;
use yang2::context::{Context, ContextFlags};
use yang2::data::{
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
        ctx.load_module(module_name, None)
            .expect("Failed to load module");
    }

    // Parse data tree in the JSON format.
    let dtree = DataTree::parse_file(
        &ctx,
        File::open("./assets/data/interfaces.json")?,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
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
