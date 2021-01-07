use yang2::context::{Context, ContextFlags};
use yang2::data::{
    Data, DataFormat, DataParserFlags, DataPrinterFlags, DataTree,
    DataValidationFlags,
};

static SEARCH_DIR: &str = "./assets/yang/";
static JSON_TREE1: &str = r###"
    {
        "ietf-interfaces:interfaces":{
            "interface": [
                {
                    "name": "eth/0/0",
                    "description": "ENG",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                }
            ],
            "interface": [
                {
                    "name": "eth/0/1",
                    "description": "MKT",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                }
            ]
        }
    }"###;
static JSON_TREE2: &str = r###"
    {
        "ietf-interfaces:interfaces":{
            "interface": [
                {
                    "name": "eth/0/0",
                    "description": "ENG",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": false
                }
            ],
            "interface": [
                {
                    "name": "eth/0/2",
                    "description": "MGMT",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                }
            ]
        }
    }"###;

fn main() -> std::io::Result<()> {
    // Initialize context.
    let ctx = Context::new(SEARCH_DIR, ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Load YANG modules.
    for module_name in &["ietf-interfaces", "iana-if-type"] {
        ctx.load_module(module_name, None)
            .expect("Failed to load module");
    }

    // Parse data trees from JSON strings.
    let dtree1 = DataTree::parse_string(
        &ctx,
        JSON_TREE1,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data tree");

    let dtree2 = DataTree::parse_string(
        &ctx,
        JSON_TREE2,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data tree");

    // Compare data trees.
    println!("Comparing data trees (JSON output):");
    let diff = dtree1.diff(&dtree2).expect("Failed to compare data trees");
    diff.print_file(
        std::io::stdout(),
        DataFormat::JSON,
        DataPrinterFlags::WITH_SIBLINGS,
    )
    .expect("Failed to print data diff");

    println!("Comparing data trees (manual iteration):");
    for (op, dnode) in diff.iter() {
        println!(" {:?}: {} ({:?})", op, dnode.path(), dnode.value());
    }

    Ok(())
}
