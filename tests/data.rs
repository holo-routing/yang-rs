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
static JSON_MERGE: &str = r###"
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
                    "name": "eth/0/1",
                    "description": "MKT",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
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

static JSON_DIFF: &str = r###"
    {
      "ietf-interfaces:interfaces": {
        "@": {
          "yang:operation": "none"
        },
        "interface": [
          {
            "name": "eth/0/0",
            "enabled": false,
            "@enabled": {
              "yang:operation": "replace",
              "yang:orig-default": false,
              "yang:orig-value": "true"
            }
          },      {
            "@": {
              "yang:operation": "delete"
            },
            "name": "eth/0/1",
            "description": "MKT",
            "type": "iana-if-type:ethernetCsmacd",
            "enabled": true
          },      {
            "@": {
              "yang:operation": "create"
            },
            "name": "eth/0/2",
            "description": "MGMT",
            "type": "iana-if-type:ethernetCsmacd",
            "enabled": true
          }
        ]
      }
    }
"###;

macro_rules! assert_data_eq {
    ($dtree1:expr, $dtree2:expr) => {
        let json1 = $dtree1
            .print_string(DataFormat::JSON, DataPrinterFlags::WITH_SIBLINGS)
            .expect("Failed to print data");
        let json2 = $dtree2
            .print_string(DataFormat::JSON, DataPrinterFlags::WITH_SIBLINGS)
            .expect("Failed to print data");

        assert_eq!(json1, json2);
    };
}

fn create_context() -> Context {
    // Initialize context.
    let ctx = Context::new(SEARCH_DIR, ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");

    // Load YANG modules.
    for module_name in &["ietf-interfaces", "iana-if-type"] {
        ctx.load_module(module_name, None)
            .expect("Failed to load module");
    }

    ctx
}

fn parse_json_data<'a>(ctx: &'a Context, string: &str) -> DataTree<'a> {
    DataTree::parse_string(
        &ctx,
        string,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data tree")
}

#[test]
fn data_find() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find("/ietf-interfaces:interfaces/interface")
            .expect("Failed to lookup data")
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']"
        ]
    );

    assert_eq!(
        dtree1
            .find("/ietf-interfaces:interfaces/interface[name='eth/0/0']/*")
            .expect("Failed to lookup data")
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/name",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/description",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/enabled",
        ]
    );
}

#[test]
fn data_find_single() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert!(dtree1
        .find_single("/ietf-interfaces:interfaces/interface")
        .is_err());
    assert!(dtree1
        .find_single("/ietf-interfaces:interfaces/interface[name='eth/0/0']")
        .is_ok());
}

#[test]
fn data_edit() {
    let ctx = create_context();
    let mut dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);

    enum Operation {
        MODIFY(&'static str, Option<&'static str>),
        DELETE(&'static str),
    }

    let changes = [
        Operation::MODIFY(
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/enabled",
            Some("false"),
        ),
        Operation::DELETE(
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']",
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
                dtree1
                    .new_path(xpath, *value)
                    .expect("Failed to edit data tree");
            }
            Operation::DELETE(xpath) => {
                dtree1.remove(xpath).expect("Failed to edit data tree")
            }
        };
    }

    assert_data_eq!(&dtree1, &dtree2);
}

#[test]
fn data_validate() {
    let ctx = create_context();
    let mut dtree1 = parse_json_data(&ctx, JSON_TREE1);

    // Mandatory node "oper-status" instance does not exist.
    // (path: /ietf-interfaces:interfaces/interface/oper-status)
    assert!(dtree1.validate(DataValidationFlags::PRESENT).is_err());
}

#[test]
fn data_duplicate() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dup = dtree1.duplicate().expect("Failed to duplicate data");

    assert_data_eq!(&dtree1, &dup);
}

#[test]
fn data_merge() {
    let ctx = create_context();
    let mut dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);
    let dtree_merge = parse_json_data(&ctx, JSON_MERGE);

    dtree1.merge(&dtree2).expect("Failed to merge data trees");
    assert_data_eq!(&dtree1, &dtree_merge);
}

#[test]
fn data_diff() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);
    let dtree_diff = parse_json_data(&ctx, JSON_DIFF);

    let diff = dtree1.diff(&dtree2).expect("Failed to compare data trees");
    assert_data_eq!(&diff, &dtree_diff);
}

#[test]
fn data_diff_apply() {
    let ctx = create_context();
    let mut dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);

    let diff = dtree1.diff(&dtree2).expect("Failed to compare data trees");
    dtree1.diff_apply(&diff).expect("Failed to apply diff");

    assert_data_eq!(&dtree1, &dtree2);
}

#[test]
fn data_iterator_traverse() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .traverse()
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/name",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/description",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/enabled",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']/name",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']/description",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']/type",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']/enabled"
        ]
    );
}

#[test]
fn data_iterator_ancestors() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find_single(
                "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
            )
            .expect("Failed to lookup data")
            .ancestors()
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces",
        ]
    );
}

#[test]
fn data_iterator_siblings() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find_single(
                "/ietf-interfaces:interfaces/interface[name='eth/0/0']"
            )
            .expect("Failed to lookup data")
            .siblings()
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec!["/ietf-interfaces:interfaces/interface[name='eth/0/1']",]
    );
}

#[test]
fn data_iterator_children() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find_single("/ietf-interfaces:interfaces")
            .expect("Failed to lookup data")
            .children()
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']",
        ]
    );
}
