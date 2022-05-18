use std::collections::BTreeSet;
use std::sync::Arc;
use yang2::context::{Context, ContextFlags};
use yang2::data::{
    Data, DataDiff, DataFormat, DataImplicitFlags, DataOperation,
    DataParserFlags, DataPrinterFlags, DataTree, DataValidationFlags,
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
    }"###;
static JSON_RDIFF: &str = r###"
    {
      "ietf-interfaces:interfaces": {
        "@": {
          "yang:operation": "none"
        },
        "interface": [
          {
            "name": "eth/0/0",
            "enabled": true,
            "@enabled": {
              "yang:operation": "replace",
              "yang:orig-default": false,
              "yang:orig-value": "false"
            }
          },      {
            "@": {
              "yang:operation": "create"
            },
            "name": "eth/0/1",
            "description": "MKT",
            "type": "iana-if-type:ethernetCsmacd",
            "enabled": true
          },      {
            "@": {
              "yang:operation": "delete"
            },
            "name": "eth/0/2",
            "description": "MGMT",
            "type": "iana-if-type:ethernetCsmacd",
            "enabled": true
          }
        ]
      }
    }"###;
static JSON_NOTIF1: &str = r###"
    {
        "ietf-isis:attempt-to-exceed-max-sequence":{
          "routing-protocol-name":"main",
          "isis-level":"level-1",
          "lsp-id":"0000.0000.0000.00-00"
        }
    }"###;
static JSON_RPC1: &str = r###"
    {
        "ietf-isis:clear-adjacency":{
          "routing-protocol-instance-name":"main"
        }
    }"###;

macro_rules! assert_data_eq {
    ($dnode1:expr, $dnode2:expr) => {
        let json1 = $dnode1
            .print_string(DataFormat::JSON, DataPrinterFlags::WITH_SIBLINGS)
            .expect("Failed to print data");
        let json2 = $dnode2
            .print_string(DataFormat::JSON, DataPrinterFlags::WITH_SIBLINGS)
            .expect("Failed to print data");

        assert_eq!(json1, json2);
    };
}

fn create_context() -> Arc<Context> {
    // Initialize context.
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");

    // Load YANG modules.
    for module_name in &[
        "iana-if-type",
        "ietf-interfaces",
        "ietf-ip",
        "ietf-routing",
        "ietf-isis",
    ] {
        ctx.load_module(module_name, None, &[])
            .expect("Failed to load module");
    }

    Arc::new(ctx)
}

fn parse_json_data(ctx: &Arc<Context>, string: &str) -> DataTree {
    DataTree::parse_string(
        &ctx,
        string,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data tree")
}

fn parse_json_diff(ctx: &Arc<Context>, string: &str) -> DataDiff {
    DataDiff::parse_string(
        &ctx,
        string,
        DataFormat::JSON,
        DataParserFlags::NO_VALIDATION,
        DataValidationFlags::empty(),
    )
    .expect("Failed to parse data diff")
}

fn parse_json_notification(ctx: &Arc<Context>, string: &str) -> DataTree {
    DataTree::parse_op_string(
        &ctx,
        string,
        DataFormat::JSON,
        DataOperation::NotificationYang,
    )
    .expect("Failed to parse YANG RPC")
}

fn parse_json_rpc(ctx: &Arc<Context>, string: &str) -> DataTree {
    DataTree::parse_op_string(
        &ctx,
        string,
        DataFormat::JSON,
        DataOperation::RpcYang,
    )
    .expect("Failed to parse YANG RPC")
}

#[test]
fn data_find_xpath() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find_xpath("/ietf-interfaces:interfaces/interface")
            .expect("Failed to lookup data")
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']"
        ]
    );

    assert_eq!(
        dtree1
            .find_xpath(
                "/ietf-interfaces:interfaces/interface[name='eth/0/0']/*"
            )
            .expect("Failed to lookup data")
            .map(|dnode| dnode.path().to_owned())
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
fn data_find_path() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert!(dtree1
        .find_path("/ietf-interfaces:interfaces/interface")
        .is_err());
    assert!(dtree1
        .find_path("/ietf-interfaces:interfaces/interface[name='eth/0/0']")
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
                    .new_path(xpath, *value, false)
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
fn data_duplicate_tree() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dup = dtree1.duplicate().expect("Failed to duplicate data tree");

    assert_data_eq!(&dtree1, &dup);
}

#[test]
fn data_duplicate_subtree() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    let dnode = dtree1
        .find_path("/ietf-interfaces:interfaces/interface[name='eth/0/0']")
        .expect("Failed to lookup data");

    // Duplicate without parents.
    let dup = dnode
        .duplicate(false)
        .expect("Failed to duplicate data subtree");
    assert_eq!(
        dup.traverse()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interface[name='eth/0/0']",
            "/ietf-interfaces:interface[name='eth/0/0']/name",
            "/ietf-interfaces:interface[name='eth/0/0']/description",
            "/ietf-interfaces:interface[name='eth/0/0']/type",
            "/ietf-interfaces:interface[name='eth/0/0']/enabled",
        ]
    );

    // Duplicate with parents.
    let dup = dnode
        .duplicate(true)
        .expect("Failed to duplicate data subtree");
    assert_eq!(
        dup.traverse()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/name",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/description",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/enabled",
        ]
    );
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
fn data_add_implicit() {
    let ctx = create_context();

    // Original data tree.
    let xpath = "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/area-address";
    let mut dtree1 = DataTree::new(&ctx);
    dtree1
        .new_path(xpath, Some("00"), false)
        .expect("Failed to edit data tree");

    // Original data tree with implicit configuration nodes added.
    let mut dtree2 = dtree1.duplicate().expect("Failed to duplicate data");
    dtree2
        .add_implicit(DataImplicitFlags::NO_STATE)
        .expect("Failed to add implicit nodes");

    // Test implicit config nodes.
    let dtree1_nodes = dtree1
        .traverse()
        .map(|dnode| dnode.path().to_owned())
        .collect::<BTreeSet<String>>();
    let dtree2_nodes = dtree2
        .traverse()
        .map(|dnode| dnode.path().to_owned())
        .collect::<BTreeSet<String>>();
    assert_eq!(
        dtree2_nodes
            .symmetric_difference(&dtree1_nodes)
            .collect::<Vec<&String>>(),
        vec![
            "/ietf-interfaces:interfaces",
            "/ietf-key-chain:key-chains",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/authentication",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/authentication/level-1",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/authentication/level-2",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/default-metric",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/default-metric/level-1",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/default-metric/level-2",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/default-metric/value",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/interfaces",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/level-type",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/lsp-mtu",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/metric-type",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/metric-type/level-1",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/metric-type/level-2",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/metric-type/value",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/mpls",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/mpls/ldp",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/overload",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/overload/status",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/preference",
            "/ietf-routing:routing/control-plane-protocols/control-plane-protocol[type='ietf-isis:isis'][name='main']/ietf-isis:isis/spf-control",
            "/ietf-routing:routing/ribs",
        ]
    );
}

#[test]
fn data_diff() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);
    let dtree_diff = parse_json_diff(&ctx, JSON_DIFF);

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
fn data_diff_reverse() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);
    let dtree_rdiff = parse_json_data(&ctx, JSON_RDIFF);

    let diff = dtree1.diff(&dtree2).expect("Failed to compare data trees");
    let rdiff = diff.reverse().expect("Failed to reverse diff");
    assert_data_eq!(&rdiff, &dtree_rdiff);
}

#[test]
fn data_iterator_traverse() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .traverse()
            .map(|dnode| dnode.path().to_owned())
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
fn data_iterator_traverse_notification() {
    let ctx = create_context();
    let dtree1 = parse_json_notification(&ctx, JSON_NOTIF1);

    assert_eq!(
        dtree1
            .traverse()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-isis:attempt-to-exceed-max-sequence",
            "/ietf-isis:attempt-to-exceed-max-sequence/routing-protocol-name",
            "/ietf-isis:attempt-to-exceed-max-sequence/isis-level",
            "/ietf-isis:attempt-to-exceed-max-sequence/lsp-id"
        ]
    );
}

#[test]
fn data_iterator_traverse_rpc() {
    let ctx = create_context();
    let dtree1 = parse_json_rpc(&ctx, JSON_RPC1);

    assert_eq!(
        dtree1
            .traverse()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-isis:clear-adjacency",
            "/ietf-isis:clear-adjacency/routing-protocol-instance-name"
        ]
    );
}

#[test]
fn data_iterator_ancestors() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find_path(
                "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
            )
            .expect("Failed to lookup data")
            .ancestors()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces",
        ]
    );
    assert_eq!(
        dtree1
            .find_path(
                "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
            )
            .expect("Failed to lookup data")
            .inclusive_ancestors()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']/type",
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
            .find_path("/ietf-interfaces:interfaces/interface[name='eth/0/0']")
            .expect("Failed to lookup data")
            .siblings()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec!["/ietf-interfaces:interfaces/interface[name='eth/0/1']",]
    );
    assert_eq!(
        dtree1
            .find_path("/ietf-interfaces:interfaces/interface[name='eth/0/0']")
            .expect("Failed to lookup data")
            .inclusive_siblings()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']"
        ]
    );
}

#[test]
fn data_iterator_children() {
    let ctx = create_context();
    let dtree1 = parse_json_data(&ctx, JSON_TREE1);

    assert_eq!(
        dtree1
            .find_path("/ietf-interfaces:interfaces")
            .expect("Failed to lookup data")
            .children()
            .map(|dnode| dnode.path().to_owned())
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface[name='eth/0/0']",
            "/ietf-interfaces:interfaces/interface[name='eth/0/1']",
        ]
    );
}

#[test]
fn data_is_default() {
    let ctx = create_context();
    let dtree2 = parse_json_data(&ctx, JSON_TREE2);

    assert_eq!(
        dtree2
            .find_path(
                "/ietf-interfaces:interfaces/interface[name='eth/0/0']/enabled"
            )
            .expect("Failed to lookup data")
            .is_default(),
        false,
    );
    assert_eq!(
        dtree2
            .find_path(
                "/ietf-interfaces:interfaces/interface[name='eth/0/2']/enabled"
            )
            .expect("Failed to lookup data")
            .is_default(),
        true,
    );
}
