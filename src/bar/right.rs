mod battery;

use amane::Service;

/// Height of the status pill, shared with the workspace strip so both pills match.
const PILL_HEIGHT: f32 = super::ITEM_HEIGHT;

/// The rings leave a small margin inside the pill.
const RING_SIZE: f32 = PILL_HEIGHT - 4.0;
const RING_THICKNESS: f32 = 5.0;

/// Size of the standalone glyphs (bluetooth, network).
const GLYPH_SIZE: f32 = 20.0;

/// Size of the glyphs drawn inside the rings.
const RING_GLYPH_SIZE: f32 = 12.0;

/// Size of the clock.
const CLOCK_SIZE: f32 = 20.0;
/// Size of the battery percentage
const BATTERY_SIZE: f32 = 16.0;

/// Space between items, and padding inside the pill.
const GAP: f32 = 16.0;
const PILL_PADDING: f32 = 12.0;

// how far one wheel step moves the volume
const STEP: i32 = 5;

/// Command opening the bluetooth manager when the icon is clicked.
const BLUETOOTH_COMMAND: &str = "blueman-manager";

pub struct Clock {
    time: String,
}

impl Service for Clock {
    fn load() -> Self {
        Self { time: now() }
    }

    fn interval() -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let time = now();
        let changed = time != self.time;

        self.time = time;

        changed
    }
}

fn now() -> String {
    amane::output("date +%H:%M").trim().to_string()
}

pub fn view(width: f32) -> amane::Rectangle {
    let clock = Clock::read();

    let mut items = amane::children![status(), battery(), crate::utils::label(&clock.time, CLOCK_SIZE),];

    // with the row's gap this keeps 15px from the screen edge
    items.push(Box::new(amane::Rectangle::new().width(15.0 - GAP).height(1.0)));

    amane::Rectangle::new()
        .width(width)
        .height(amane::Parent)
        .padding(amane::Padding {
            right: super::HORIZONTAL_PADDING,
            ..amane::Padding::default()
        })
        .child(
            amane::Row::new(items)
                .width(amane::Parent)
                .height(amane::Parent)
                .gap(GAP)
                .justify(amane::End)
                .align(amane::Center),
        )
}

// bluetooth, volume and network in one capsule
fn status() -> amane::Rectangle {
    let row = amane::Row::new(amane::children![bluetooth(), volume(), network()])
        .gap(GAP)
        .align(amane::Center);

    // two glyphs and the ring, separated by two gaps, plus padding on each side
    let width = GLYPH_SIZE * 2.0 + RING_SIZE + GAP * 2.0 + PILL_PADDING * 2.0;

    crate::utils::pill(width, PILL_HEIGHT, crate::theme::BACKGROUND_HIGHLIGHT)
        .align_child(amane::Center, amane::Center)
        .child(row)
}

fn bluetooth() -> amane::Rectangle {
    let bluetooth = amane::Bluetooth::read();

    let connected = bluetooth.devices().iter().any(|device| device.connected());

    let (glyph, color) = if !bluetooth.powered() {
        ("\u{f00b2}", crate::theme::TEXT_MUTED)
    } else if connected {
        ("\u{f00b1}", crate::theme::BORDER)
    } else {
        ("\u{f00af}", crate::theme::TEXT)
    };

    amane::Rectangle::new()
        .width(GLYPH_SIZE)
        .height(GLYPH_SIZE)
        .align_child(amane::Center, amane::Center)
        .cursor(amane::Pointer)
        .on_click(|_| amane::spawn(BLUETOOTH_COMMAND))
        .child(crate::utils::icon(glyph, GLYPH_SIZE, color))
}

// a ring showing the volume, scroll to change it, click to mute
fn volume() -> amane::Rectangle {
    let audio = amane::Audio::read();

    let (glyph, color) = if audio.muted() {
        ("\u{f075f}", crate::theme::TEXT_MUTED)
    } else {
        ("\u{f057e}", crate::theme::TEXT)
    };

    let value = f32::from(audio.volume()) / 100.0;

    let ring = crate::utils::ring(RING_SIZE, RING_THICKNESS, value, color, crate::theme::BACKGROUND);

    let inner = amane::Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .align_child(amane::Center, amane::Center)
        .child(crate::utils::icon(glyph, RING_GLYPH_SIZE, color));

    amane::Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .cursor(amane::Pointer)
        .on_click(|_| amane::Audio::toggle_mute())
        .on_scroll(scroll_volume)
        .child(amane::Stack::new(amane::children![ring, inner,]))
}

fn network() -> amane::Text {
    let network = amane::Network::read();

    let (glyph, color) = match network.link() {
        amane::Link::Wifi => (wifi_glyph(network.strength()), crate::theme::TEXT),
        amane::Link::Wired => ("\u{f0200}", crate::theme::TEXT),
        amane::Link::Other => ("\u{f0318}", crate::theme::TEXT),
        amane::Link::Offline => ("\u{f092e}", crate::theme::TEXT_MUTED),
    };

    crate::utils::icon(glyph, GLYPH_SIZE, color)
}

fn wifi_glyph(strength: u8) -> &'static str {
    match strength {
        76.. => "\u{f0928}",
        51..=75 => "\u{f0925}",
        26..=50 => "\u{f0922}",
        _ => "\u{f091f}",
    }
}

// a ring with the battery glyph inside, then the percentage; none without a battery
fn battery() -> amane::Row {
    let battery = battery::Battery::read();

    let percent = battery.percent();

    // orange only when it needs attention, charging shows through its glyph
    let color = battery_color(percent, battery.charging());

    let ring = crate::utils::ring(
        RING_SIZE,
        RING_THICKNESS,
        f32::from(percent) / 100.0,
        color,
        crate::theme::BACKGROUND_HIGHLIGHT,
    );

    let inner = amane::Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .align_child(amane::Center, amane::Center)
        .child(crate::utils::icon(
            battery_glyph(percent, battery.charging()),
            RING_GLYPH_SIZE,
            color,
        ));

    amane::Row::new(amane::children![
        amane::Stack::new(amane::children![ring, inner]),
        crate::utils::label(&format!("{percent}%"), BATTERY_SIZE),
    ])
    .gap(6.0)
    .align(amane::Center)
}

// one glyph per 10%
fn battery_glyph(percent: u8, charging: bool) -> &'static str {
    if charging {
        return "\u{f06a5}";
    }

    match percent {
        91.. => "\u{f0079}",
        81..=90 => "\u{f0082}",
        71..=80 => "\u{f0081}",
        61..=70 => "\u{f0080}",
        51..=60 => "\u{f007f}",
        41..=50 => "\u{f007e}",
        31..=40 => "\u{f007d}",
        21..=30 => "\u{f007c}",
        11..=20 => "\u{f007b}",
        _ => "\u{f007a}",
    }
}

// wheel up is a negative y, which turns the volume up
fn scroll_volume(scroll: amane::Scroll) {
    let volume = i32::from(amane::Audio::read().volume());
    let change = if scroll.y < 0.0 { STEP } else { -STEP };

    amane::Audio::set_volume((volume + change).clamp(0, 100) as u8);
}

/// Green while charging, otherwise TEXT fading to red as the charge drops from 50% to 0%.
fn battery_color(percent: u8, charging: bool) -> amane::Color {
    if charging {
        return crate::theme::SUCCESS;
    }

    // 0.0 at 50% and above, 1.0 at 0%
    let danger = (1.0 - f32::from(percent) / 50.0).clamp(0.0, 1.0);

    <amane::Color as amane::Blend>::blend(crate::theme::TEXT, crate::theme::DANGER, danger)
}
