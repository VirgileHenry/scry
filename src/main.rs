mod bar;
mod calendar;
mod date;
mod disk_usage;
mod theme;

mod utils;

use amane::{App, Service};

fn main() {
    init_logging();

    /* Load the calendar links before the first update */
    amane_calendar::CalendarService::write().add_calendar(
        "Proton Calendar".to_string(),
        include_str!("calendars/main_calendar.secret").trim().to_string(),
    );
    amane_calendar::CalendarService::write().add_calendar(
        "Birthdays".to_string(),
        include_str!("calendars/bday_calendar.secret").trim().to_string(),
    );

    amane_disk_usage::DiskUsageService::write().watch_directory("/nix/store".to_string(), "/nix/store");
    amane_disk_usage::DiskUsageService::write().watch_directory("~/Downloads".to_string(), "/home/eclipse/Downloads");
    amane_disk_usage::DiskUsageService::write().watch_directory("~/dev/rust".to_string(), "/home/eclipse/dev/rust");
    amane_disk_usage::DiskUsageService::write().watch_directory("~/dev/projects".to_string(), "/home/eclipse/dev/projects");

    App::new()
        .window_per_monitor(bar::view)
        .window_per_monitor(date::view)
        .window_per_monitor(calendar::view)
        .window_per_monitor(disk_usage::view)
        .run();
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
