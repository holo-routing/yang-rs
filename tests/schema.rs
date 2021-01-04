use libyang2_sys as ffi;
use yang2::context::{Context, ContextFlags};
use yang2::schema::{SchemaNodeCommon, SchemaNodeKind};

static SEARCH_DIR: &str = "./assets/yang/";

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

#[test]
fn schema_find() {
    let ctx = create_context();
    let snode = ctx.traverse().next().unwrap();

    assert_eq!(
        snode
            .find("/ietf-interfaces:interfaces/*")
            .expect("Failed to lookup schema data")
            .map(|dnode| dnode.path().unwrap())
            .collect::<Vec<String>>(),
        vec!["/ietf-interfaces:interfaces/interface"]
    );

    assert_eq!(
        snode
            .find("/ietf-interfaces:interfaces/interface/*")
            .expect("Failed to lookup schema data")
            .map(|dnode| dnode.path().unwrap())
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
fn schema_find_single() {
    let ctx = create_context();
    let snode_top = ctx.traverse().next().unwrap();

    assert!(snode_top
        .find_single("/ietf-interfaces:interfaces/interface/*")
        .is_err());
    assert!(snode_top
        .find_single("/ietf-interfaces:interfaces/interface")
        .is_ok());
}

#[test]
fn schema_iterator_traverse() {
    let ctx = create_context();
    let snode_top = ctx.traverse().next().unwrap();

    assert_eq!(
        snode_top
            .traverse()
            .map(|snode| snode.path().unwrap())
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
        ]
    );
}

#[test]
fn schema_iterator_ancestors() {
    let ctx = create_context();
    let snode_top = ctx.traverse().next().unwrap();

    assert_eq!(
        snode_top
            .find_single("/ietf-interfaces:interfaces/interface/statistics/discontinuity-time")
            .expect("Failed to lookup schema data")
            .ancestors()
            .map(|snode| snode.path().unwrap())
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
    let snode_top = ctx.traverse().next().unwrap();

    assert_eq!(
        snode_top
            .find_single("/ietf-interfaces:interfaces/interface/name")
            .expect("Failed to lookup schema data")
            .siblings()
            .map(|snode| snode.path().unwrap())
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
    let snode_top = ctx.traverse().next().unwrap();

    assert_eq!(
        snode_top
            .find_single("/ietf-interfaces:interfaces/interface/statistics")
            .expect("Failed to lookup schema data")
            .children()
            .map(|snode| snode.path().unwrap())
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
    let snode_top = ctx.traverse().next().unwrap();

    let snode = snode_top
        .find_single("/ietf-interfaces:interfaces/interface/enabled")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.name(), "enabled");
    assert!(snode.description().is_some());
    assert!(snode.reference().is_some());
    if let SchemaNodeKind::Leaf(sleaf) = snode.kind() {
        assert_eq!(sleaf.config(), true);
        assert_eq!(sleaf.mandatory(), false);
        assert_eq!(sleaf.default(), Some("true"));
        assert_eq!(sleaf.base_type(), ffi::LY_DATA_TYPE::LY_TYPE_BOOL);
        assert!(sleaf.units().is_none());
        assert!(sleaf.musts().next().is_none());
        assert!(sleaf.whens().next().is_none());
    }

    let snode = snode_top
        .find_single("/ietf-interfaces:interfaces/interface")
        .expect("Failed to lookup schema node");
    assert_eq!(snode.name(), "interface");
    assert!(snode.description().is_some());
    assert!(snode.reference().is_none());
    if let SchemaNodeKind::List(slist) = snode.kind() {
        assert_eq!(slist.config(), true);
        assert_eq!(slist.mandatory(), false);
        assert_eq!(slist.keyless(), false);
        assert_eq!(slist.user_ordered(), false);
        assert_eq!(slist.min_elements(), None);
        assert_eq!(slist.max_elements(), None);
        assert!(slist.musts().next().is_none());
        assert!(slist.whens().next().is_none());
        assert!(slist.actions().next().is_none());
        assert!(slist.notifications().next().is_none());
    }
}
