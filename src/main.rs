mod bar;
mod calendar;
mod date;
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

    App::new()
        .window_per_monitor(bar::view)
        .window_per_monitor(date::view)
        .window_per_monitor(calendar::view)
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
