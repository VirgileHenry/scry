use amane::*;
use amane_calendar::icalendar::{Component, EventLike};

/// The number of calendar events shown per calendar.
const SHOWN: usize = 4;

/// Distance between the top of the screen and the first card
const TOP_MARGIN: i32 = 48;
/// Distance between the left of the screen and the left of the card
const LEFT_MARGIN: i32 = 24;

/// Ticks once per minute, so "starts in / ends in" never goes stale.
/// A view only has to read it to be redrawn on every new minute.
struct Minute(i64);

impl Service for Minute {
    fn new() -> Self {
        Self(0)
    }

    fn listen() {
        use chrono::Timelike;

        loop {
            /* wake just after the next minute starts */
            let now = chrono::Local::now();
            let into_minute = u64::from(now.second()) * 1000 + u64::from(now.timestamp_subsec_millis().min(999));
            std::thread::sleep(std::time::Duration::from_millis(60_001 - into_minute));

            Self::write().0 += 1;
        }
    }
}

/// Holds the events we want to display
struct DisplayedEvents {
    /// All events we are currently in
    current: Vec<amane_calendar::Event>,
    /// All events scheduled today but we are not currently in
    today: Vec<amane_calendar::Event>,
    /// Next events after today
    incoming: Vec<amane_calendar::Event>,
}

impl DisplayedEvents {
    pub fn from_events(events: impl Iterator<Item = amane_calendar::Event>, max_incoming: usize) -> Self {
        let now = chrono::Utc::now();
        let today_date = now.date_naive();

        let mut current = Vec::new();
        let mut today = Vec::new();
        let mut incoming = Vec::new();

        for event in events {
            match event.schedule {
                amane_calendar::Schedule::AllDay { start, .. } => {
                    if start == today_date {
                        current.push(event.clone());
                    } else {
                        incoming.push(event.clone());
                    }
                }
                amane_calendar::Schedule::Schedule { start, .. } => {
                    if start <= now {
                        current.push(event.clone());
                    } else if start.date_naive() == today_date {
                        today.push(event.clone());
                    } else {
                        incoming.push(event.clone());
                    }
                }
            }

            if incoming.len() >= max_incoming {
                break;
            }
        }

        Self {
            current,
            today,
            incoming,
        }
    }

    fn view(&self) -> Vec<Box<dyn amane::Widget>> {
        let event_count = self.current.len() + self.today.len() + self.incoming.len() + 4;
        let mut widgets = Vec::with_capacity(event_count);

        if !self.current.is_empty() {
            widgets.extend(self.current());
        }
        if !self.today.is_empty() {
            widgets.extend(self.today());
        }
        if !self.incoming.is_empty() {
            widgets.extend(self.incoming());
        }

        widgets
    }

    fn current(&self) -> impl Iterator<Item = Box<dyn amane::Widget>> {
        Self::category_view("Current:", &self.current)
    }

    fn today(&self) -> impl Iterator<Item = Box<dyn amane::Widget>> {
        Self::category_view("Today:", &self.today)
    }

    fn incoming(&self) -> impl Iterator<Item = Box<dyn amane::Widget>> {
        Self::category_view("Next Events:", &self.incoming)
    }

    fn category_view(category_name: &str, events: &[amane_calendar::Event]) -> impl Iterator<Item = Box<dyn amane::Widget>> {
        let section_title = std::iter::once(Box::new(
            Text::new(category_name)
                .size(40.0)
                .weight(Weight::Medium)
                .color(Color::from(crate::theme::BACKGROUND_HIGHLIGHT)),
        ) as Box<dyn amane::Widget>);
        let events = events
            .iter()
            .map(|ev| crate::utils::indent(20.0, event_card(ev)))
            .map(|card| Box::new(card) as Box<dyn amane::Widget>);

        section_title.chain(events)
    }
}

pub fn view(_monitor: &Monitor) -> LayerWindow {
    let _minute = Minute::read();
    let service = amane_calendar::CalendarService::read();

    let last_update = service.last_update().format("%H:%M");
    let last_update_widget = Text::new(format!("(Updated at {})", last_update))
        .size(16.0)
        .weight(amane::Weight::Light)
        .color(crate::theme::BACKGROUND_HIGHLIGHT);

    let events = DisplayedEvents::from_events(service.raw_incoming_events(), SHOWN);
    let mut content = events.view();
    content.push(Box::new(last_update_widget) as Box<dyn amane::Widget>);

    LayerWindow::new()
        .width(amane::WindowSize::Full)
        .height(amane::WindowSize::Full)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .margin(Margin {
            top: TOP_MARGIN,
            left: LEFT_MARGIN,
            ..amane::Margin::default()
        })
        .layer(Layer::Bottom)
        .click_through()
        .namespace("calendar")
        .child(amane::Column::new(content).gap(12.0))
}

fn event_card(event: &amane_calendar::Event) -> Column {
    let title = event.event.get_summary().unwrap_or("(no title)");

    let title = Text::new(title).size(22.0).color(crate::theme::BACKGROUND);
    let schedule = Text::new(schedule_label(&event.schedule))
        .size(22.0)
        .weight(amane::Weight::Light)
        .color(crate::theme::BACKGROUND_HIGHLIGHT);

    let event_header = Row::new(children![title, schedule])
        .width(Parent)
        .gap(8.0)
        .align(amane::Align::Start);

    let mut desc_content = Vec::new();

    if let Some(desc) = event.event.get_description() {
        add_description_line(&mut desc_content, desc);
    }
    let attendees = event.event.get_attendees();
    if !attendees.is_empty() {
        let attendees: Vec<&str> = attendees
            .iter()
            .map(|at| match at.cn.as_ref() {
                Some(name) => name.as_str(),
                None => at.cal_address.as_str(),
            })
            .collect();
        let attendees = attendees.join(", ");
        add_description_line(&mut desc_content, &format!("☺ {attendees}"));
    }
    if let Some(loc) = event.event.get_location() {
        add_description_line(&mut desc_content, &format!("@ {loc}"));
    }

    let description = crate::utils::indent(10.0, Column::new(desc_content).width(Parent));

    Column::new(children![event_header, description]).width(Parent).gap(2.0)
}

fn add_description_line(desc: &mut Vec<Box<dyn amane::Widget>>, text: &str) {
    desc.push(Box::new(
        Text::new(text)
            .size(18.0)
            .weight(amane::Weight::Light)
            .color(crate::theme::BACKGROUND),
    ));
}

/// "10:00 - 11:00 (ends in 25 mins)" / "14:00 - 15:00 (starts in 2h)".
fn schedule_label(schedule: &amane_calendar::Schedule) -> String {
    let now = chrono::Utc::now();

    match schedule {
        amane_calendar::Schedule::AllDay { start, end } => match end {
            Some(end) => {
                let now = now.with_timezone(&chrono::Local).date_naive();
                if *start <= now {
                    let remaining = *end - now;
                    format!(
                        "{} - {} ({} remaining)",
                        start.format("%-d %b"),
                        end.format("%-d %b"),
                        most_significant(remaining)
                    )
                } else {
                    let incoming = *start - now;
                    format!(
                        "{} - {} (in {})",
                        start.format("%-d %b"),
                        end.format("%-d %b"),
                        most_significant(incoming)
                    )
                }
            }
            None => {
                let now = now.with_timezone(&chrono::Local).date_naive();
                let incoming = *start - now;
                format!("{} (in {})", start.format("%-d %b"), most_significant(incoming))
            }
        },
        amane_calendar::Schedule::Schedule { start, end } => match end {
            Some(end) => {
                if *start <= now {
                    let remaining = *end - now;
                    format!(
                        "{} - {} ({} remaining)",
                        start.with_timezone(&chrono::Local).format("%H:%M"),
                        end.with_timezone(&chrono::Local).format("%H:%M"),
                        most_significant(remaining)
                    )
                } else {
                    let incoming = *start - now;
                    let remaining_days = if incoming.num_days() > 0 {
                        format!("{}, ", start.format("%-d %b"))
                    } else {
                        "".to_string()
                    };
                    format!(
                        "{}{} - {} (in {})",
                        remaining_days,
                        start.with_timezone(&chrono::Local).format("%H:%M"),
                        end.with_timezone(&chrono::Local).format("%H:%M"),
                        most_significant(incoming)
                    )
                }
            }
            None => {
                let incoming = *start - now;
                let remaining_days = if incoming.num_days() > 0 {
                    format!("{}, ", start.format("%-d %b"))
                } else {
                    "".to_string()
                };
                format!(
                    "{}{} (in {})",
                    remaining_days,
                    start.with_timezone(&chrono::Local).format("%H:%M"),
                    most_significant(incoming)
                )
            }
        },
    }
}

fn most_significant(d: chrono::TimeDelta) -> String {
    let (n, unit) = match d.num_minutes() {
        m if m < 60 => (m, "min"),
        m if m < 60 * 24 => ((m + 30) / 60, "hour"),
        m => (m / (60 * 24), "day"),
    };
    format!("{n} {unit}{}", if n == 1 { "" } else { "s" })
}
