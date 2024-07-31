use std::fs::File;
use yang3::context::{Context, ContextFlags};
use yang3::data::{
    Data, DataFormat, DataParserFlags, DataPrinterFlags, DataTree,
    DataValidationFlags,
};

static SEARCH_DIR: &str = "./assets/yang/";

enum Operation {
    MODIFY(&'static str, Option<&'static str>),
    DELETE(&'static str),
}

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

    // Parse data tree from JSON file.
    let mut dtree = DataTree::parse_file(
        &ctx,
        File::open("./assets/data/interfaces.json")?,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data tree");

    // Modify data tree.
    let changes = [
        Operation::DELETE(
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
        ),
        Operation::MODIFY(
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']/description",
            Some("HR"),
        ),
        Operation::MODIFY(
            "/ietf-interfaces:interfaces/interface[name='eth/0/2']/description",
            Some("MGMT"),
        ),
        Operation::MODIFY(
            "/ietf-interfaces:interfaces/interface[name='eth/0/2']/type",
            Some("iana-if-type:ethernetCsmacd"),
        ),
        Operation::MODIFY(
            "/ietf-interfaces:interfaces/interface[name='eth/0/2']/enabled",
            Some("true"),
        ),
    ];
    for change in &changes {
        match change {
            Operation::MODIFY(xpath, value) => {
                dtree
                    .new_path(xpath, *value, false)
                    .expect("Failed to edit data tree");
            }
            Operation::DELETE(xpath) => {
                dtree.remove(xpath).expect("Failed to edit data tree")
            }
        };
    }

    // Print the modified data tree.
    dtree
        .print_file(
            std::io::stdout(),
            DataFormat::JSON,
            DataPrinterFlags::WD_ALL | DataPrinterFlags::WITH_SIBLINGS,
        )
        .expect("Failed to print data tree");

    Ok(())
}
