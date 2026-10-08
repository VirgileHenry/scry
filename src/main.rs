mod bar;
mod date;
mod theme;
mod utils;

use amane::App;

fn main() {
    init_logging();

    App::new().window_per_monitor(bar::view).window_per_monitor(date::view).run();
}

fn init_logging() {
    #[cfg(debug_assertions)]
    let log_level = "debug";
    #[cfg(not(debug_assertions))]
    let log_level = "info";

    let directives = format!("warn,amane=info,mockingbird={log_level}");

    let filter = tracing_subscriber::EnvFilter::new(directives);
    let builder = tracing_subscriber::fmt().with_env_filter(filter);

    builder.init();
}
