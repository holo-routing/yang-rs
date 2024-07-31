use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use yang3::context::{Context, ContextFlags};
use yang3::data::{Data, DataDiffFlags, DataTree, DataValidationFlags};

static SEARCH_DIR: &str = "./assets/yang/";

fn data_generate(ctx: &Context, interfaces: u32) -> DataTree {
    let mut dtree = DataTree::new(ctx);

    for i in 1..=interfaces {
        let changes = [
            (format!("/ietf-interfaces:interfaces/interface[name='eth{}']", i), None),
            (format!("/ietf-interfaces:interfaces/interface[name='eth{}']/type", i), Some("iana-if-type:ethernetCsmacd")),
            (format!("/ietf-interfaces:interfaces/interface[name='eth{}']/enabled", i), Some("true")),
        ];

        for (xpath, value) in &changes {
            dtree
                .new_path(xpath, *value, false)
                .expect("Failed to edit data tree");
        }
    }

    dtree
}

fn criterion_benchmark(c: &mut Criterion) {
    let tree_sizes = [
        1 * 1024,
        2 * 1024,
        4 * 1024,
        8 * 1024,
        16 * 1024,
        32 * 1024,
        64 * 1024,
    ];

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

    // Prepare DataTree.diff() benchmark.
    let mut group = c.benchmark_group("DataTree.diff() / tree size");
    for size in &tree_sizes {
        // Create artificial data trees.
        let dtree = data_generate(&ctx, *size);
        let dtree_base = data_generate(&ctx, *size + 1024);

        // Run benchmark.
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    dtree
                        .diff(&dtree_base, DataDiffFlags::empty())
                        .expect("Failed to compare data trees");
                });
            },
        );
    }
    group.finish();

    // Prepare DataTree.find() benchmark.
    let mut group = c.benchmark_group("DataTree.find() / tree size");
    for size in &tree_sizes {
        // Create artificial data tree.
        let dtree = data_generate(&ctx, *size);

        // Run benchmark.
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    for dnode in dtree.traverse() {
                        let path = dnode.path();
                        dtree.find_path(&path).expect("Failed to find data");
                    }
                });
            },
        );
    }
    group.finish();

    // Prepare DataTree.validate() benchmark.
    let mut group = c.benchmark_group("DataTree.validate() / tree size");
    for size in &tree_sizes {
        // Create artificial data tree.
        let mut dtree = data_generate(&ctx, *size);

        // Run benchmark.
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    dtree
                        .validate(
                            DataValidationFlags::NO_STATE
                                | DataValidationFlags::PRESENT,
                        )
                        .expect("Failed to validate data tree")
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
