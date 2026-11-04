use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use yang5::context::{Context, ContextFlags};
use yang5::data::DataFormat;
use yang5::iter::IterSchemaFlags;
use yang5::schema::{
    DataValueType, SchemaInputFormat, SchemaModule, SchemaNode, SchemaNodeKind,
    SchemaPathFormat, SchemaStmtMust,
};

static SEARCH_DIR: &str = "./assets/yang/";
static YANG_LIBRARY_FILE: &str = "./assets/data/lib.json";

static JSON_YANG_LIBRARY: &str = r###"
    {
      "ietf-yang-library:yang-library": {
        "module-set": [
          {
            "name": "complete",
            "module": [
              {
                "name": "iana-if-type",
                "revision": "2017-01-19",
                "namespace": "urn:ietf:params:xml:ns:yang:iana-if-type"
              },
              {
                "name": "ietf-interfaces",
                "revision": "2018-02-20",
                "namespace": "urn:ietf:params:xml:ns:yang:ietf-interfaces",
                "feature": [
                  "arbitrary-names",
                  "pre-provisioning",
                  "if-mib"
                ]
              }
            ],
            "import-only-module": []
          }
        ],
        "schema": [
          {
            "name": "complete",
            "module-set": [
              "complete"
            ]
          }
        ],
        "content-id": "34"
      },
      "ietf-yang-library:modules-state": {
        "module-set-id": "34",
        "module": [
          {
            "name": "iana-if-type",
            "revision": "2017-01-19",
            "namespace": "urn:ietf:params:xml:ns:yang:iana-if-type",
            "conformance-type": "implement"
          },
          {
            "name": "ietf-interfaces",
            "revision": "2018-02-20",
            "namespace": "urn:ietf:params:xml:ns:yang:ietf-interfaces",
            "feature": [
              "arbitrary-names",
              "pre-provisioning",
              "if-mib"
            ],
            "conformance-type": "implement"
          }
        ]
      }
    }"###;

fn create_context() -> Context {
    // Initialize context.
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");

    // Load YANG modules.
    ctx.load_module("ietf-interfaces", None, &["pre-provisioning"])
        .expect("Failed to load module");
    ctx.load_module("iana-if-type", None, &[])
        .expect("Failed to load module");
    ctx.load_module("ietf-key-chain", None, &["hex-key-string"])
        .expect("Failed to load module");
    ctx.load_module("ietf-routing", None, &[])
        .expect("Failed to load module");
    ctx.load_module("ietf-mpls-ldp", None, &[])
        .expect("Failed to load module");

    ctx
}

// Returns the paths of the nodes accessed by the first "when" condition of a
// schema node.
fn when_atoms(snode: &SchemaNode<'_>) -> BTreeSet<String> {
    let when = snode.whens().next().unwrap();
    when.atoms(snode)
        .unwrap()
        .iter()
        .map(|atom| atom.path(SchemaPathFormat::DATA))
        .collect()
}

fn create_test_context() -> Context {
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");
    ctx.load_module("yang-rs-test", None, &[])
        .expect("Failed to load module");
    ctx
}

#[test]
fn schema_feature_value() {
    let ctx = create_context();
    let module = ctx.get_module_latest("ietf-interfaces").unwrap();
    assert_eq!(module.feature_value("pre-provisioning"), Ok(true));
    assert_eq!(module.feature_value("if-mib"), Ok(false));
    assert!(module.feature_value("blabla").is_err());
}

#[test]
fn schema_find_xpath() {
    let ctx = create_context();

    assert_eq!(
        ctx.find_xpath("/ietf-interfaces:interfaces/*")
            .expect("Failed to lookup schema data")
            .map(|dnode| dnode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec!["/ietf-interfaces:interfaces/interface"]
    );

    assert_eq!(
        ctx.find_xpath("/ietf-interfaces:interfaces/interface/*")
            .expect("Failed to lookup schema data")
            .map(|dnode| dnode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface/name",
            "/ietf-interfaces:interfaces/interface/description",
            "/ietf-interfaces:interfaces/interface/type",
            "/ietf-interfaces:interfaces/interface/enabled",
            "/ietf-interfaces:interfaces/interface/oper-status",
            "/ietf-interfaces:interfaces/interface/last-change",
            "/ietf-interfaces:interfaces/interface/phys-address",
            "/ietf-interfaces:interfaces/interface/higher-layer-if",
            "/ietf-interfaces:interfaces/interface/lower-layer-if",
            "/ietf-interfaces:interfaces/interface/speed",
            "/ietf-interfaces:interfaces/interface/statistics",
        ]
    );
}

#[test]
fn schema_find_path() {
    let ctx = create_context();

    assert!(ctx
        .find_path("/ietf-interfaces:interfaces/interface/*")
        .is_err());
    assert!(ctx
        .find_path("/ietf-interfaces:interfaces/interface")
        .is_ok());
}

#[test]
fn schema_iterator_traverse() {
    let ctx = create_context();

    assert_eq!(
        ctx
            .traverse()
            .filter(|snode| snode.module().name() == "ietf-interfaces")
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces",
            "/ietf-interfaces:interfaces/interface",
            "/ietf-interfaces:interfaces/interface/name",
            "/ietf-interfaces:interfaces/interface/description",
            "/ietf-interfaces:interfaces/interface/type",
            "/ietf-interfaces:interfaces/interface/enabled",
            "/ietf-interfaces:interfaces/interface/oper-status",
            "/ietf-interfaces:interfaces/interface/last-change",
            "/ietf-interfaces:interfaces/interface/phys-address",
            "/ietf-interfaces:interfaces/interface/higher-layer-if",
            "/ietf-interfaces:interfaces/interface/lower-layer-if",
            "/ietf-interfaces:interfaces/interface/speed",
            "/ietf-interfaces:interfaces/interface/statistics",
            "/ietf-interfaces:interfaces/interface/statistics/discontinuity-time",
            "/ietf-interfaces:interfaces/interface/statistics/in-octets",
            "/ietf-interfaces:interfaces/interface/statistics/in-unicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/in-broadcast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/in-multicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/in-discards",
            "/ietf-interfaces:interfaces/interface/statistics/in-errors",
            "/ietf-interfaces:interfaces/interface/statistics/in-unknown-protos",
            "/ietf-interfaces:interfaces/interface/statistics/out-octets",
            "/ietf-interfaces:interfaces/interface/statistics/out-unicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/out-broadcast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/out-multicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/out-discards",
            "/ietf-interfaces:interfaces/interface/statistics/out-errors",
            "/ietf-interfaces:interfaces-state",
            "/ietf-interfaces:interfaces-state/interface",
            "/ietf-interfaces:interfaces-state/interface/name",
            "/ietf-interfaces:interfaces-state/interface/type",
            "/ietf-interfaces:interfaces-state/interface/oper-status",
            "/ietf-interfaces:interfaces-state/interface/last-change",
            "/ietf-interfaces:interfaces-state/interface/phys-address",
            "/ietf-interfaces:interfaces-state/interface/higher-layer-if",
            "/ietf-interfaces:interfaces-state/interface/lower-layer-if",
            "/ietf-interfaces:interfaces-state/interface/speed",
            "/ietf-interfaces:interfaces-state/interface/statistics",
            "/ietf-interfaces:interfaces-state/interface/statistics/discontinuity-time",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-octets",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-unicast-pkts",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-broadcast-pkts",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-multicast-pkts",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-discards",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-errors",
            "/ietf-interfaces:interfaces-state/interface/statistics/in-unknown-protos",
            "/ietf-interfaces:interfaces-state/interface/statistics/out-octets",
            "/ietf-interfaces:interfaces-state/interface/statistics/out-unicast-pkts",
            "/ietf-interfaces:interfaces-state/interface/statistics/out-broadcast-pkts",
            "/ietf-interfaces:interfaces-state/interface/statistics/out-multicast-pkts",
            "/ietf-interfaces:interfaces-state/interface/statistics/out-discards",
            "/ietf-interfaces:interfaces-state/interface/statistics/out-errors"
        ]
    );
}

#[test]
fn schema_iterator_ancestors() {
    let ctx = create_context();

    assert_eq!(
        ctx
            .find_path("/ietf-interfaces:interfaces/interface/statistics/discontinuity-time")
            .expect("Failed to lookup schema data")
            .ancestors()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface/statistics",
            "/ietf-interfaces:interfaces/interface",
            "/ietf-interfaces:interfaces",
        ]
    );
    assert_eq!(
        ctx
            .find_path("/ietf-interfaces:interfaces/interface/statistics/discontinuity-time")
            .expect("Failed to lookup schema data")
            .inclusive_ancestors()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface/statistics/discontinuity-time",
            "/ietf-interfaces:interfaces/interface/statistics",
            "/ietf-interfaces:interfaces/interface",
            "/ietf-interfaces:interfaces",
        ]
    );
}

#[test]
fn schema_iterator_siblings() {
    let ctx = create_context();

    assert_eq!(
        ctx.find_path("/ietf-interfaces:interfaces/interface/name")
            .expect("Failed to lookup schema data")
            .siblings()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface/description",
            "/ietf-interfaces:interfaces/interface/type",
            "/ietf-interfaces:interfaces/interface/enabled",
            "/ietf-interfaces:interfaces/interface/oper-status",
            "/ietf-interfaces:interfaces/interface/last-change",
            "/ietf-interfaces:interfaces/interface/phys-address",
            "/ietf-interfaces:interfaces/interface/higher-layer-if",
            "/ietf-interfaces:interfaces/interface/lower-layer-if",
            "/ietf-interfaces:interfaces/interface/speed",
            "/ietf-interfaces:interfaces/interface/statistics",
        ]
    );
    assert_eq!(
        ctx.find_path("/ietf-interfaces:interfaces/interface/name")
            .expect("Failed to lookup schema data")
            .inclusive_siblings()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface/name",
            "/ietf-interfaces:interfaces/interface/description",
            "/ietf-interfaces:interfaces/interface/type",
            "/ietf-interfaces:interfaces/interface/enabled",
            "/ietf-interfaces:interfaces/interface/oper-status",
            "/ietf-interfaces:interfaces/interface/last-change",
            "/ietf-interfaces:interfaces/interface/phys-address",
            "/ietf-interfaces:interfaces/interface/higher-layer-if",
            "/ietf-interfaces:interfaces/interface/lower-layer-if",
            "/ietf-interfaces:interfaces/interface/speed",
            "/ietf-interfaces:interfaces/interface/statistics",
        ]
    );
}

#[test]
fn schema_iterator_children() {
    let ctx = create_context();

    assert_eq!(
        ctx
            .find_path("/ietf-interfaces:interfaces/interface/statistics")
            .expect("Failed to lookup schema data")
            .children()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces/interface/statistics/discontinuity-time",
            "/ietf-interfaces:interfaces/interface/statistics/in-octets",
            "/ietf-interfaces:interfaces/interface/statistics/in-unicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/in-broadcast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/in-multicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/in-discards",
            "/ietf-interfaces:interfaces/interface/statistics/in-errors",
            "/ietf-interfaces:interfaces/interface/statistics/in-unknown-protos",
            "/ietf-interfaces:interfaces/interface/statistics/out-octets",
            "/ietf-interfaces:interfaces/interface/statistics/out-unicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/out-broadcast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/out-multicast-pkts",
            "/ietf-interfaces:interfaces/interface/statistics/out-discards",
            "/ietf-interfaces:interfaces/interface/statistics/out-errors",
        ]
    );

    assert_eq!(
        ctx.find_path("/ietf-routing:routing/ribs/rib")
            .expect("Failed to lookup schema data")
            .children()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-routing:routing/ribs/rib/name",
            "/ietf-routing:routing/ribs/rib/address-family",
            "/ietf-routing:routing/ribs/rib/routes",
            "/ietf-routing:routing/ribs/rib/description"
        ]
    );

    assert_eq!(
        ctx.find_path("/ietf-routing:routing/ribs/rib")
            .expect("Failed to lookup schema data")
            .all_children()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-routing:routing/ribs/rib/name",
            "/ietf-routing:routing/ribs/rib/address-family",
            "/ietf-routing:routing/ribs/rib/routes",
            "/ietf-routing:routing/ribs/rib/description",
            "/ietf-routing:routing/ribs/rib/active-route"
        ]
    );
}

#[test]
fn schema_iterator_children2() {
    let ctx = create_context();

    assert_eq!(
        ctx.find_path("/ietf-key-chain:key-chains/key-chain/key/key-string")
            .expect("Failed to lookup schema data")
            .children2(IterSchemaFlags::empty())
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-key-chain:key-chains/key-chain/key/key-string/keystring",
            "/ietf-key-chain:key-chains/key-chain/key/key-string/hexadecimal-string"
        ]
    );

    assert_eq!(
        ctx.find_path("/ietf-key-chain:key-chains/key-chain/key/key-string")
            .expect("Failed to lookup schema data")
            .children2(IterSchemaFlags::NO_CHOICE)
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        Vec::<String>::new()
    );

    assert_eq!(
        ctx.find_path("/ietf-key-chain:key-chains/key-chain/key")
            .expect("Failed to lookup schema data")
            .children2(IterSchemaFlags::empty())
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-key-chain:key-chains/key-chain/key/key-id",
            "/ietf-key-chain:key-chains/key-chain/key/lifetime",
            "/ietf-key-chain:key-chains/key-chain/key/crypto-algorithm",
            "/ietf-key-chain:key-chains/key-chain/key/key-string",
            "/ietf-key-chain:key-chains/key-chain/key/send-lifetime-active",
            "/ietf-key-chain:key-chains/key-chain/key/accept-lifetime-active"
        ]
    );

    assert_eq!(
        ctx.find_path("/ietf-key-chain:key-chains/key-chain/key")
            .expect("Failed to lookup schema data")
            .children2(IterSchemaFlags::INTO_NP_CONT)
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-key-chain:key-chains/key-chain/key/key-id",
            "/ietf-key-chain:key-chains/key-chain/key/lifetime/send-accept-lifetime/always",
            "/ietf-key-chain:key-chains/key-chain/key/lifetime/send-accept-lifetime/start-date-time",
            "/ietf-key-chain:key-chains/key-chain/key/lifetime/send-accept-lifetime/no-end-time",
            "/ietf-key-chain:key-chains/key-chain/key/lifetime/send-accept-lifetime/duration",
            "/ietf-key-chain:key-chains/key-chain/key/lifetime/send-accept-lifetime/end-date-time",
            "/ietf-key-chain:key-chains/key-chain/key/crypto-algorithm",
            "/ietf-key-chain:key-chains/key-chain/key/key-string/keystring",
            "/ietf-key-chain:key-chains/key-chain/key/key-string/hexadecimal-string",
            "/ietf-key-chain:key-chains/key-chain/key/send-lifetime-active",
            "/ietf-key-chain:key-chains/key-chain/key/accept-lifetime-active"
        ]
    );

    assert_eq!(
        ctx.find_path("/ietf-routing:routing/ribs/rib")
            .expect("Failed to lookup schema data")
            .children2(IterSchemaFlags::empty())
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-routing:routing/ribs/rib/name",
            "/ietf-routing:routing/ribs/rib/address-family",
            "/ietf-routing:routing/ribs/rib/routes",
            "/ietf-routing:routing/ribs/rib/description",
            "/ietf-routing:routing/ribs/rib/active-route"
        ]
    );
}

#[test]
fn schema_iterator_top_level_nodes() {
    let ctx = create_context();

    assert_eq!(
        ctx.get_module_latest("ietf-interfaces")
            .expect("Failed to lookup schema module")
            .top_level_nodes(IterSchemaFlags::empty())
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-interfaces:interfaces",
            "/ietf-interfaces:interfaces-state"
        ]
    );

    assert_eq!(
        ctx.get_module_latest("ietf-mpls-ldp")
            .expect("Failed to lookup schema module")
            .top_level_nodes(IterSchemaFlags::empty())
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<String>>(),
        vec![
            "/ietf-mpls-ldp:mpls-ldp-clear-peer",
            "/ietf-mpls-ldp:mpls-ldp-clear-hello-adjacency",
            "/ietf-mpls-ldp:mpls-ldp-clear-peer-statistics",
            "/ietf-mpls-ldp:mpls-ldp-peer-event",
            "/ietf-mpls-ldp:mpls-ldp-hello-adjacency-event",
            "/ietf-mpls-ldp:mpls-ldp-fec-event"
        ]
    );
}

#[test]
fn schema_node_default_value() {
    let ctx = create_context();

    let snode = ctx
        .find_path("/ietf-interfaces:interfaces/interface/enabled")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.default_value_canonical(), Ok(Some("true".to_owned())));

    // Leaves without a default value.
    let snode = ctx
        .find_path("/ietf-interfaces:interfaces/interface/description")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.default_value_canonical(), Ok(None));
    let snode = ctx
        .find_path("/ietf-interfaces:interfaces/interface/type")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.default_value_canonical(), Ok(None));
}

#[test]
fn schema_leaf_type_restrictions() {
    let ctx = create_test_context();

    // Ranges.
    let snode = ctx.find_path("/yang-rs-test:types/counter").unwrap();
    let ltype = snode.leaf_type().unwrap();
    let range = ltype.range().unwrap();
    assert_eq!(range.intervals(), [(-10, -1), (1, 10)]);
    assert_eq!(range.error_msg(), Some("Out of range."));
    assert!(ltype.length().is_none());
    assert_eq!(ltype.fraction_digits(), None);
    let snode = ctx.find_path("/yang-rs-test:types/big").unwrap();
    assert!(snode.leaf_type().unwrap().range().is_none());
    let snode = ctx.find_path("/yang-rs-test:types/ratio").unwrap();
    let ltype = snode.leaf_type().unwrap();
    assert_eq!(ltype.fraction_digits(), Some(2));
    assert_eq!(ltype.range().unwrap().intervals(), [(50, 10000)]);

    // Lengths and patterns.
    let snode = ctx.find_path("/yang-rs-test:types/name").unwrap();
    let ltype = snode.leaf_type().unwrap();
    assert_eq!(ltype.length().unwrap().intervals(), [(1, 8)]);
    assert!(ltype.range().is_none());
    let patterns = ltype
        .patterns()
        .map(|pattern| {
            (
                pattern.expr().to_owned(),
                pattern.is_inverted(),
                pattern.error_msg().map(str::to_owned),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        patterns,
        [
            ("[a-z]+".to_owned(), false, None),
            (
                "x.*".to_owned(),
                true,
                Some("Can't start with x.".to_owned())
            ),
        ]
    );
    let snode = ctx.find_path("/yang-rs-test:types/data").unwrap();
    let ltype = snode.leaf_type().unwrap();
    assert_eq!(ltype.length().unwrap().intervals(), [(4, 4)]);
    assert_eq!(ltype.patterns().count(), 0);

    // Enumerations and bits.
    let snode = ctx.find_path("/yang-rs-test:types/level").unwrap();
    let ltype = snode.leaf_type().unwrap();
    let enums = ltype
        .enums()
        .map(|item| {
            (
                item.name().to_owned(),
                item.value(),
                item.description().map(str::to_owned),
                item.is_status_deprecated(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        enums,
        [
            ("low".to_owned(), 1, Some("Low level.".to_owned()), false),
            ("high".to_owned(), 10, None, true),
        ]
    );
    assert_eq!(ltype.bits().count(), 0);
    let snode = ctx.find_path("/yang-rs-test:types/flags").unwrap();
    let ltype = snode.leaf_type().unwrap();
    let bits = ltype
        .bits()
        .map(|item| {
            (
                item.name().to_owned(),
                item.position(),
                item.description().map(str::to_owned),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        bits,
        [
            ("up".to_owned(), 0, None),
            ("down".to_owned(), 3, Some("Down.".to_owned())),
        ]
    );
    assert_eq!(ltype.enums().count(), 0);

    // Unions.
    let snode = ctx.find_path("/yang-rs-test:types/mixed").unwrap();
    let ltype = snode.leaf_type().unwrap();
    let members = ltype.union_types().collect::<Vec<_>>();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].base_type(), DataValueType::Uint8);
    assert_eq!(members[0].typedef_name().as_deref(), Some("percent"));
    assert_eq!(members[0].range().unwrap().intervals(), [(0, 100)]);
    assert_eq!(members[1].base_type(), DataValueType::Enum);
    assert_eq!(members[1].enums().next().unwrap().name(), "auto");
}

#[test]
fn schema_node_validate_value() {
    let ctx = create_test_context();

    let snode = ctx.find_path("/yang-rs-test:types/name").unwrap();
    assert!(snode.validate_value("abc").is_ok());
    assert!(snode.validate_value("ab1").is_err());
    let error = snode.validate_value("xyz").unwrap_err();
    assert_eq!(error.msg.as_deref(), Some("Can't start with x."));
    assert!(snode.validate_value("a\0b").is_err());

    let snode = ctx.find_path("/yang-rs-test:types/counter").unwrap();
    assert!(snode.validate_value("5").is_ok());
    let error = snode.validate_value("0").unwrap_err();
    assert_eq!(error.msg.as_deref(), Some("Out of range."));

    // Leafrefs are checked against their real type only.
    let snode = ctx.find_path("/yang-rs-test:types/peer").unwrap();
    assert!(snode.validate_value("abc").is_ok());
    assert!(snode.validate_value("ab1").is_err());

    let snode = ctx.find_path("/yang-rs-test:types/codes").unwrap();
    assert!(snode.validate_value("5").is_ok());
    assert!(snode.validate_value("abc").is_err());

    // Other kinds of nodes (case, choice and container) hold no values.
    let snode = ctx.find_path("/yang-rs-test:types/text").unwrap();
    let kinds = snode
        .ancestors()
        .map(|ancestor| {
            let error = ancestor.validate_value("5").unwrap_err();
            assert_eq!(
                error.msg.as_deref(),
                Some("Schema node isn't a leaf(-list).")
            );
            ancestor.kind()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            SchemaNodeKind::Case,
            SchemaNodeKind::Choice,
            SchemaNodeKind::Container
        ]
    );
}

#[test]
fn schema_node_leafref_targets() {
    let ctx = create_test_context();

    let paths = |path: &str| {
        ctx.find_path(path)
            .unwrap()
            .leafref_targets()
            .iter()
            .map(|snode| snode.path(SchemaPathFormat::DATA))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        paths("/yang-rs-test:types/peer"),
        ["/yang-rs-test:types/name"]
    );
    assert_eq!(
        paths("/yang-rs-test:types/limit"),
        ["/yang-rs-test:types/counter"]
    );
    assert!(paths("/yang-rs-test:types/name").is_empty());
}

#[test]
fn schema_must_and_when_statements() {
    let ctx = create_test_context();

    let snode = ctx.find_path("/yang-rs-test:constraints/limit").unwrap();
    let musts = snode.musts().unwrap().collect::<Vec<_>>();
    assert_eq!(musts.len(), 2);
    assert_eq!(musts[0].condition(), ". <= 50");
    assert_eq!(musts[0].error_msg(), Some("Too high."));
    assert_eq!(musts[1].condition(), ". < ../max");

    // Nodes accessed by the conditions.
    let atoms = |must: &SchemaStmtMust<'_>| {
        must.atoms(&snode)
            .unwrap()
            .iter()
            .map(|atom| atom.path(SchemaPathFormat::DATA))
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(
        atoms(&musts[0]),
        BTreeSet::from(["/yang-rs-test:constraints/limit".to_owned()])
    );
    assert!(atoms(&musts[1]).contains("/yang-rs-test:constraints/max"));

    // Context nodes of "when" conditions.
    let snode = ctx.find_path("/yang-rs-test:constraints/ospf").unwrap();
    let when = snode.whens().next().unwrap();
    assert_eq!(
        when.condition(),
        "derived-from-or-self(../protocol, 't:ospf')"
    );
    assert_eq!(when.context_node(), Some(snode.clone()));
    let snode = ctx.find_path("/yang-rs-test:constraints/burst").unwrap();
    let case = snode.ancestors().next().unwrap();
    let when = case.whens().next().unwrap();
    assert_eq!(when.condition(), "level = 'high'");
    assert_eq!(
        when.context_node().unwrap().path(SchemaPathFormat::DATA),
        "/yang-rs-test:constraints"
    );

    // Nodes accessed by "when" conditions, from each kind of context node: the
    // node itself, the closest data node containing a case, and the root.
    let snode = ctx.find_path("/yang-rs-test:constraints/ospf").unwrap();
    assert_eq!(
        when_atoms(&snode),
        BTreeSet::from([
            "/yang-rs-test:constraints".to_owned(),
            "/yang-rs-test:constraints/protocol".to_owned(),
        ])
    );
    assert_eq!(
        when_atoms(&case),
        BTreeSet::from(["/yang-rs-test:constraints/level".to_owned()])
    );
    let snode = ctx.find_path("/yang-rs-test:top-burst").unwrap();
    let case = snode.ancestors().next().unwrap();
    assert_eq!(case.whens().next().unwrap().context_node(), None);
    assert_eq!(
        when_atoms(&case),
        BTreeSet::from([
            "/yang-rs-test:constraints".to_owned(),
            "/yang-rs-test:constraints/level".to_owned(),
        ])
    );
}

#[test]
fn schema_when_atoms_in_augments() {
    let ctx = create_test_context();

    // The context node of the condition belongs to another module, so nothing
    // but the augmenting node tells what the unprefixed name refers to.
    SchemaModule::parse_string(
        &ctx,
        r#"
        module yang-rs-test-augment {
          yang-version 1.1;
          namespace "urn:yang-rs-test-augment";
          prefix a;

          import yang-rs-test {
            prefix t;
          }

          augment "/t:constraints" {
            leaf mode {
              type string;
            }
          }
          augment "/t:constraints" {
            when "mode = 'on'";
            leaf flag {
              type boolean;
            }
          }
        }
        "#,
        SchemaInputFormat::YANG,
    )
    .expect("Failed to parse module");

    let snode = ctx
        .find_path("/yang-rs-test:constraints/yang-rs-test-augment:flag")
        .unwrap();
    let when = snode.whens().next().unwrap();
    assert_eq!(when.context_node().unwrap().module().name(), "yang-rs-test");
    assert!(when_atoms(&snode)
        .contains("/yang-rs-test:constraints/yang-rs-test-augment:mode"));
}

#[test]
fn schema_atoms_in_rpc_output() {
    let ctx = create_test_context();

    // The input and the output have a leaf of the same name, so the atoms tell
    // which of the two a condition was evaluated against.
    SchemaModule::parse_string(
        &ctx,
        r#"
        module yang-rs-test-rpc {
          yang-version 1.1;
          namespace "urn:yang-rs-test-rpc";
          prefix r;

          rpc run {
            input {
              leaf mode {
                type string;
              }
              leaf in-flag {
                when "../mode = 'on'";
                must "../mode = 'on'";
                type boolean;
              }
            }
            output {
              leaf mode {
                type string;
              }
              leaf out-flag {
                when "../mode = 'on'";
                must "../mode = 'on'";
                type boolean;
              }
              anyxml out-any {
                when "../mode = 'on'";
              }
            }
          }
        }
        "#,
        SchemaInputFormat::YANG,
    )
    .expect("Failed to parse module");

    // The paths of the input and output nodes only differ in this format.
    fn log_paths(atoms: Vec<SchemaNode<'_>>) -> BTreeSet<String> {
        atoms
            .iter()
            .map(|atom| atom.path(SchemaPathFormat::LOG))
            .collect()
    }
    let expected = |leaf: &str| {
        BTreeSet::from([
            "/yang-rs-test-rpc:run".to_owned(),
            format!("/yang-rs-test-rpc:run/{}", leaf),
        ])
    };

    let rpc = ctx.find_path("/yang-rs-test-rpc:run").unwrap();
    let (mut input, _) = rpc.input().unwrap();
    let (output, _) = rpc.output().unwrap();
    let output = output.collect::<Vec<_>>();

    let snode = input.find(|snode| snode.name() == "in-flag").unwrap();
    let when = snode.whens().next().unwrap();
    let must = snode.musts().unwrap().next().unwrap();
    assert_eq!(
        log_paths(when.atoms(&snode).unwrap()),
        expected("input/mode")
    );
    assert_eq!(
        log_paths(must.atoms(&snode).unwrap()),
        expected("input/mode")
    );

    let snode = output.iter().find(|s| s.name() == "out-flag").unwrap();
    let when = snode.whens().next().unwrap();
    let must = snode.musts().unwrap().next().unwrap();
    assert_eq!(
        log_paths(when.atoms(snode).unwrap()),
        expected("output/mode")
    );
    assert_eq!(
        log_paths(must.atoms(snode).unwrap()),
        expected("output/mode")
    );

    let snode = output.iter().find(|s| s.name() == "out-any").unwrap();
    let when = snode.whens().next().unwrap();
    assert_eq!(
        log_paths(when.atoms(snode).unwrap()),
        expected("output/mode")
    );
}

#[test]
fn schema_identities() {
    let ctx = create_test_context();

    let snode = ctx.find_path("/yang-rs-test:types/protocol").unwrap();
    let ltype = snode.leaf_type().unwrap();
    let bases = ltype.identity_bases().collect::<Vec<_>>();
    assert_eq!(bases.len(), 1);
    let base = &bases[0];
    assert_eq!(base.name(), "routing-protocol");
    assert_eq!(base.module().name(), "yang-rs-test");

    let mut derived = base.derived().collect::<Vec<_>>();
    derived.sort_by(|a, b| a.name().cmp(b.name()));
    assert_eq!(
        derived.iter().map(|ident| ident.name()).collect::<Vec<_>>(),
        ["ospf", "rip"]
    );
    let ospf = &derived[0];
    assert_eq!(ospf.description(), Some("OSPF."));
    assert!(ospf.is_enabled());
    assert!(!ospf.is_status_deprecated());
    assert_eq!(ospf.derived().count(), 2);

    // Identities disabled by "if-feature".
    let rip = &derived[1];
    assert!(!rip.is_enabled());
    assert!(rip.is_status_deprecated());
    assert_ne!(ospf, rip);
}

#[test]
fn schema_node_attributes() {
    let ctx = create_context();

    let snode = ctx
        .find_path("/ietf-interfaces:interfaces/interface/enabled")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.kind(), SchemaNodeKind::Leaf);
    assert_eq!(snode.name(), "enabled");
    assert!(snode.description().is_some());
    assert!(snode.reference().is_some());
    assert_eq!(snode.is_config(), true);
    assert_eq!(snode.is_state(), false);
    assert_eq!(snode.is_mandatory(), false);
    assert_eq!(snode.default_value_canonical(), Ok(Some("true".to_owned())));
    assert_eq!(snode.leaf_type().unwrap().base_type(), DataValueType::Bool);
    assert!(snode.units().is_none());
    assert!(snode.musts().unwrap().count() == 0);
    assert!(snode.whens().count() == 0);
    assert_eq!(snode.is_status_current(), true);
    assert_eq!(snode.is_status_deprecated(), false);
    assert_eq!(snode.is_status_obsolete(), false);

    let snode = ctx
        .find_path("/ietf-interfaces:interfaces/interface")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.kind(), SchemaNodeKind::List);
    assert_eq!(snode.name(), "interface");
    assert!(snode.description().is_some());
    assert!(snode.reference().is_none());
    assert_eq!(snode.is_config(), true);
    assert_eq!(snode.is_state(), false);
    assert_eq!(snode.is_mandatory(), false);
    assert_eq!(snode.is_keyless_list(), false);
    assert_eq!(snode.is_user_ordered(), false);
    assert_eq!(snode.min_elements(), None);
    assert_eq!(snode.max_elements(), None);
    assert!(snode.musts().unwrap().count() == 0);
    assert!(snode.whens().count() == 0);
    assert!(snode.actions().count() == 0);
    assert!(snode.notifications().count() == 0);
    assert_eq!(snode.is_status_current(), true);
    assert_eq!(snode.is_status_deprecated(), false);
    assert_eq!(snode.is_status_obsolete(), false);

    let snode = ctx
        .find_path("/ietf-interfaces:interfaces-state/interface")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.kind(), SchemaNodeKind::List);
    assert_eq!(snode.name(), "interface");
    assert!(snode.description().is_some());
    assert!(snode.reference().is_none());
    assert_eq!(snode.is_config(), false);
    assert_eq!(snode.is_state(), true);
    assert_eq!(snode.is_mandatory(), false);
    assert_eq!(snode.is_keyless_list(), false);
    // TODO: this is wrong, report back to upstream.
    assert_eq!(snode.is_user_ordered(), true);
    assert_eq!(snode.min_elements(), None);
    assert_eq!(snode.max_elements(), None);
    assert!(snode.musts().unwrap().count() == 0);
    assert!(snode.whens().count() == 0);
    assert!(snode.actions().count() == 0);
    assert!(snode.notifications().count() == 0);
    assert_eq!(snode.is_status_current(), false);
    assert_eq!(snode.is_status_deprecated(), true);
    assert_eq!(snode.is_status_obsolete(), false);
}

#[test]
fn test_create_context_from_yang_library_path() {
    let ctx = Context::new_from_yang_library_file(
        YANG_LIBRARY_FILE,
        DataFormat::JSON,
        SEARCH_DIR,
        ContextFlags::empty(),
    )
    .expect("Failed to create context");

    let module_names = ctx
        .modules(true)
        .map(|m| m.name().to_string())
        .collect::<BTreeSet<String>>();
    let expected = BTreeSet::from([
        "ietf-interfaces".to_string(),
        "iana-if-type".to_string(),
    ]);

    assert_eq!(module_names, expected);
}

#[test]
fn test_create_context_from_yang_library_str() {
    let ctx = Context::new_from_yang_library_str(
        JSON_YANG_LIBRARY,
        DataFormat::JSON,
        SEARCH_DIR,
        ContextFlags::empty(),
    )
    .expect("Failed to create context");

    let module_names = ctx
        .modules(true)
        .map(|m| m.name().to_string())
        .collect::<BTreeSet<String>>();
    let expected = BTreeSet::from([
        "ietf-interfaces".to_string(),
        "iana-if-type".to_string(),
    ]);

    assert_eq!(module_names, expected);
}

#[test]
fn test_extensions_uncompiled_modules() {
    let ctx = create_context();
    // ietf-yang-metadata is an internal module and it's not compiled
    let module = ctx.get_module_latest("ietf-yang-metadata").unwrap();
    let extensions = module.extensions().collect::<Vec<_>>();
    assert_eq!(extensions.len(), 0);
}

#[test]
fn schema_module_imports() {
    let mut ctx = create_context();

    // Test a module that has imports
    let module = ctx.get_module_latest("ietf-interfaces").unwrap();
    let imports: Vec<_> = module.imports().collect();

    // ietf-interfaces imports only ietf-yang-types
    assert_eq!(imports.len(), 1);

    // Test the ietf-yang-types import
    let yang_types_import = &imports[0];
    assert_eq!(yang_types_import.name(), "ietf-yang-types");
    assert_eq!(yang_types_import.prefix(), "yang");
    assert_eq!(yang_types_import.reference(), None);

    // Test imported module access
    let imported_module = yang_types_import.module();
    assert_eq!(imported_module.name(), "ietf-yang-types");
    assert_eq!(
        imported_module.namespace(),
        "urn:ietf:params:xml:ns:yang:ietf-yang-types"
    );
    assert_eq!(imported_module.reference(), None);

    // Test a module with no imports (like ietf-restconf)
    let module = ctx
        .load_module("ietf-restconf", None, &[])
        .expect("Failed to load module");
    let module_imports: Vec<_> = module.imports().collect();
    assert_eq!(module_imports.len(), 0);
}

#[test]
fn test_parse_schema_from_str() {
    let ctx = create_context();
    let valid_schema_str = std::fs::read_to_string(
        Path::new(SEARCH_DIR).join("ietf-routing@2018-01-25.yang"),
    )
    .expect("failed to read schema file");
    let invalid_schema = std::fs::read_to_string(YANG_LIBRARY_FILE)
        .expect("failed to read data file");

    let valid_module = SchemaModule::parse_string(
        &ctx,
        &valid_schema_str,
        SchemaInputFormat::YANG,
    );
    let invalid_module = SchemaModule::parse_string(
        &ctx,
        &invalid_schema,
        SchemaInputFormat::YANG,
    );

    assert!(valid_module.is_ok_and(|x| x.name() == "ietf-routing"));
    assert!(invalid_module.is_err());
}

#[test]
fn context_module_importer() {
    let mut ctx = Context::new(
        ContextFlags::NO_YANGLIBRARY | ContextFlags::DISABLE_SEARCHDIR_CWD,
    )
    .expect("Failed to create context");
    let requests = Arc::new(Mutex::new(vec![]));
    let requests_cb = requests.clone();
    ctx.set_module_importer(move |request| {
        requests_cb
            .lock()
            .unwrap()
            .push(request.module_name.to_owned());
        match request.module_name {
            "yang-rs-test" => std::fs::read_to_string(
                Path::new(SEARCH_DIR).join("yang-rs-test.yang"),
            )
            .ok()
            .map(|data| (SchemaInputFormat::YANG, data)),
            _ => None,
        }
    });
    ctx.load_module("yang-rs-test", None, &[])
        .expect("Failed to load module");
    assert!(ctx.load_module("missing", None, &[]).is_err());
    let requests = requests.lock().unwrap().clone();
    assert!(requests.contains(&"yang-rs-test".to_owned()));
    assert!(requests.contains(&"missing".to_owned()));

    // Panics are contained.
    ctx.set_module_importer(|_| panic!("importer panic"));
    assert!(ctx.load_module("other", None, &[]).is_err());

    // The closure is freed when replaced.
    let token = Arc::new(());
    let token_cb = token.clone();
    ctx.set_module_importer(move |_| {
        let _ = &token_cb;
        None
    });
    assert_eq!(Arc::strong_count(&token), 2);
    ctx.unset_embedded_modules();
    assert_eq!(Arc::strong_count(&token), 1);
}

#[test]
fn test_load_modules() {
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY)
        .expect("Failed to create context");
    ctx.set_searchdir(SEARCH_DIR)
        .expect("Failed to set YANG search directory");
    ctx.load_modules(&[
        ("ietf-interfaces", None, &["pre-provisioning"]),
        ("ietf-routing", None, &[]),
    ])
    .expect("Failed to load modules");

    assert!(ctx.get_module_latest("ietf-interfaces").is_some());
    assert!(ctx.get_module_latest("ietf-routing").is_some());
}
