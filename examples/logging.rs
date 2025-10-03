use log::LevelFilter;
use yang3::context::{Context, ContextFlags};

fn main() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::max())
        .init();
    let mut ctx = Context::new(ContextFlags::NO_YANGLIBRARY).unwrap();
    ctx.set_log_level_debug();
    ctx.init_default_logger().unwrap();
    ctx.set_searchdir("./assets/yang/").unwrap();
    // When loading modules, we should see some logs
    let _module = ctx.load_module("ietf-isis", None, &[]).unwrap();
}
