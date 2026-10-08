use std::time::Duration;

use amane::{Column, End, Layer, LayerWindow, Margin, Monitor, Parent, Service, Text, Weight, children};

const TIME_SIZE: f32 = 160.0;
const DATE_SIZE: f32 = 32.0;

/// Distance from the top right corner of the usable area.
const MARGIN: i32 = 40;

/// Time and date shown on the wallpaper.
pub struct DesktopClock {
    time: String,
    date: String,
}

impl Service for DesktopClock {
    fn load() -> Self {
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

pub fn view(_monitor: &Monitor) -> LayerWindow {
    let clock = DesktopClock::read();

    LayerWindow::new()
        .width(amane::WindowSize::Full)
        .height(amane::WindowSize::Full)
        .margin(Margin {
            top: MARGIN,
            right: MARGIN,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Bottom)
        .click_through()
        .child(
            Column::new(children![
                Text::new(&clock.time)
                    .size(TIME_SIZE)
                    .weight(Weight::ExtraBold)
                    .color(crate::theme::BACKGROUND),
                Text::new(&clock.date).size(DATE_SIZE).color(crate::theme::BACKGROUND),
            ])
            .width(Parent)
            .height(Parent)
            .gap(-TIME_SIZE * 0.2)
            .align(End),
        )
}
