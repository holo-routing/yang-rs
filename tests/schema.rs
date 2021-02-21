use yang2::context::{Context, ContextFlags};
use yang2::schema::{DataValueType, SchemaNodeKind, SchemaPathFormat};

static SEARCH_DIR: &str = "./assets/yang/";

fn create_context() -> Context {
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

    ctx
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
    assert_eq!(snode.is_mandatory(), false);
    assert_eq!(snode.default_value(), Some("true"));
    assert_eq!(snode.base_type(), Some(DataValueType::Bool));
    assert!(snode.units().is_none());
    assert!(snode.musts().unwrap().count() == 0);
    assert!(snode.whens().count() == 0);

    let snode = ctx
        .find_path("/ietf-interfaces:interfaces/interface")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.kind(), SchemaNodeKind::List);
    assert_eq!(snode.name(), "interface");
    assert!(snode.description().is_some());
    assert!(snode.reference().is_none());
    assert_eq!(snode.is_config(), true);
    assert_eq!(snode.is_mandatory(), false);
    assert_eq!(snode.is_keyless_list(), false);
    assert_eq!(snode.is_user_ordered(), false);
    assert_eq!(snode.min_elements(), None);
    assert_eq!(snode.max_elements(), None);
    assert!(snode.musts().unwrap().count() == 0);
    assert!(snode.whens().count() == 0);
    assert!(snode.actions().unwrap().count() == 0);
    assert!(snode.notifications().unwrap().count() == 0);
}
