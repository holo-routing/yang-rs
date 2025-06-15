use std::collections::BTreeSet;
use yang3::context::{Context, ContextFlags};
use yang3::data::DataFormat;
use yang3::iter::IterSchemaFlags;
use yang3::schema::{
    DataValue, DataValueType, SchemaNodeKind, SchemaPathFormat,
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
    assert_eq!(snode.default_value_canonical(), Some("true"));
    assert_eq!(snode.default_value(), Some(DataValue::Bool(true)));
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
fn ext_yang_data() {
    let mut ctx = create_context();

    let module = ctx
        .load_module("ietf-restconf", None, &[])
        .expect("Failed to load module");

    assert_eq!(
        module
            .extensions()
            .filter_map(|ext| ext.argument())
            .collect::<Vec<String>>(),
        ["yang-errors", "yang-api"]
    );

    // yang-errors
    let ext = module
        .extensions()
        .find(|ext| ext.argument().as_deref() == Some("yang-errors"))
        .expect("Failed to find the \"yang-api\" extension instance");

    let dtree = ext.new_inner("errors").expect("Failed to create data");

    assert_eq!(
        dtree
            .traverse()
            .map(|dnode| dnode.path())
            .collect::<Vec<String>>(),
        vec!["/ietf-restconf:errors"]
    );

    // yang-api
    let ext = module
        .extensions()
        .find(|ext| ext.argument().as_deref() == Some("yang-api"))
        .expect("Failed to find the \"yang-api\" extension instance");

    let dtree = ext
        .new_path("/ietf-restconf:restconf/data", None, false)
        .expect("Failed to create data")
        .unwrap();

    assert_eq!(
        dtree
            .traverse()
            .map(|dnode| dnode.path())
            .collect::<Vec<String>>(),
        vec!["/ietf-restconf:restconf", "/ietf-restconf:restconf/data"]
    );
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
