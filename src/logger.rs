use std::env;

use env_logger::{Builder, Env};

pub fn init(prefix: &str) {
    let prefix = prefix.to_uppercase();
    init_human(prefix.as_str())
}

fn new_env(prefix: &str) -> Env<'static> {
    let filter_env = format!("{prefix}_LOG");
    let style_env = format!("{prefix}_LOG_STYLE");
    Env::new().filter(filter_env).write_style(style_env)
}

fn init_human(prefix: &str) {
    human_panic::setup_panic!();

    Builder::from_env(new_env(prefix))
        .format_timestamp_millis()
        .init();
}
