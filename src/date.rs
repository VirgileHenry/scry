use std::time::Duration;

const TIME_SIZE: f32 = 150.0;
const DATE_SIZE: f32 = 32.0;

const DIGIT_WIDTH: f32 = TIME_SIZE * 0.54;
const COLON_WIDTH: f32 = TIME_SIZE * 0.30;

/// Distance from the top right corner of the usable area.
const MARGIN: i32 = 40;

/// Time and date shown on the wallpaper.
pub struct DesktopClock {
    time: String,
    date: String,
}

impl amane::Service for DesktopClock {
    fn new() -> Self {
        let (time, date) = now();
        Self { time, date }
    }

    fn listen() {
        loop {
            /* wake just after the next second starts, so the display never skips a second */
            let millis = chrono::Local::now().timestamp_subsec_millis().min(999);
            std::thread::sleep(Duration::from_millis(u64::from(1000 - millis)));

            let (time, date) = now();
            let mut clock = Self::write();
            clock.time = time;
            clock.date = date;
        }
    }
}

fn now() -> (String, String) {
    let now = chrono::Local::now();
    (now.format("%H:%M:%S").to_string(), now.format("%A %-d %B %Y").to_string())
}

pub fn view(_monitor: &amane::Monitor) -> amane::LayerWindow {
    use amane::Service;
    let clock = DesktopClock::read();

    amane::LayerWindow::new()
        .width(amane::WindowSize::Full)
        .height(amane::WindowSize::Full)
        .margin(amane::Margin {
            top: MARGIN,
            right: MARGIN,
            bottom: 0,
            left: 0,
        })
        .layer(amane::Layer::Bottom)
        .click_through()
        .child(
            amane::Column::new(amane::children![
                time_row(&clock.time),
                crate::utils::label(&clock.date, DATE_SIZE).color(crate::theme::BACKGROUND),
            ])
            .width(amane::Parent)
            .height(amane::Parent)
            .align(amane::End),
        )
}

fn time_row(time: &str) -> amane::Row {
    let chars = time.chars().map(|c| {
        let width = if c == ':' { COLON_WIDTH } else { DIGIT_WIDTH };
        amane::Rectangle::new()
            .width(width)
            .height(TIME_SIZE)
            .child(
                crate::utils::label(&c.to_string(), TIME_SIZE)
                    .weight(amane::Weight::SemiBold)
                    .color(crate::theme::BACKGROUND),
            )
            .align_child(amane::Center, amane::Center)
    });
    let content = chars.map(|c| Box::new(c) as Box<dyn amane::Widget>).collect();
    amane::Row::new(content)
}
