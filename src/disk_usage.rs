/// Distance between the bottom of the screen and the last line
const BOTTOM_MARGIN: i32 = 48;
/// Distance between the left of the screen and the left of the content
const LEFT_MARGIN: i32 = 24;

/// Indent of the watched directories under their file system
const DIR_INDENT: f32 = 20.0;
/// Width of a file system bar, directory bars are shortened by the indent so both end at the same x
const BAR_WIDTH: f32 = 420.0;
const BAR_HEIGHT: f32 = 8.0;

/// Free space: the empty part of every bar
const TRACK: amane::Color = amane::Color::rgba(128, 128, 128, 60);

/// One color per watched directory, reused in order if there are more directories than colors
const DIR_COLORS: [amane::Color; 6] = [
    amane::Color::rgb(122, 162, 247),
    amane::Color::rgb(158, 206, 106),
    amane::Color::rgb(224, 175, 104),
    amane::Color::rgb(187, 154, 247),
    amane::Color::rgb(247, 118, 142),
    amane::Color::rgb(125, 207, 255),
];

pub fn view(_monitor: &amane::Monitor) -> amane::LayerWindow {
    use amane::Service;

    let service = amane_disk_usage::DiskUsageService::read();

    let file_systems: Vec<&amane_disk_usage::FileSystemUsage> = service.file_systems().collect();

    /* Give every directory its color, then attach it to the deepest mount point containing it */
    let mut dirs_per_fs: Vec<Vec<(&amane_disk_usage::DirectoryUsage, amane::Color)>> =
        (0..file_systems.len()).map(|_| Vec::new()).collect();
    for (i, dir) in service.directories().enumerate() {
        let color = DIR_COLORS[i % DIR_COLORS.len()];
        let fs = file_systems
            .iter()
            .enumerate()
            .filter(|(_, fs)| dir.directory().starts_with(fs.mount_point()))
            .max_by_key(|(_, fs)| fs.mount_point().components().count());
        if let Some((index, _)) = fs {
            dirs_per_fs[index].push((dir, color));
        }
    }

    let mut content: Vec<Box<dyn amane::Widget>> = Vec::new();
    content.push(Box::new(
        amane::Text::new("File Systems:")
            .size(40.0)
            .weight(amane::Weight::Medium)
            .color(amane::Color::from(crate::theme::BACKGROUND_HIGHLIGHT)),
    ));
    for (fs, dirs) in file_systems.iter().zip(&dirs_per_fs) {
        content.push(Box::new(crate::utils::indent(20.0, fs_card(fs, dirs))));
    }

    let last_update = match service.last_update() {
        Some(last_update) => format!("(Updated at {})", last_update.naive_local().format("%H:%M")),
        None => "Updating...".to_string(),
    };
    content.push(Box::new(
        amane::Text::new(last_update)
            .size(16.0)
            .weight(amane::Weight::Light)
            .color(crate::theme::BACKGROUND_HIGHLIGHT),
    ));

    amane::LayerWindow::new()
        .width(amane::WindowSize::Full)
        .height(amane::WindowSize::Full)
        .anchor_vertical(amane::Vertical::Bottom)
        .anchor_horizontal(amane::Horizontal::Left)
        .margin(amane::Margin {
            bottom: BOTTOM_MARGIN,
            left: LEFT_MARGIN,
            ..amane::Margin::default()
        })
        .layer(amane::Layer::Bottom)
        .click_through()
        .namespace("disk-usage")
        .child(
            amane::Column::new(content)
                .height(amane::Parent)
                .justify(amane::Justify::End)
                .gap(12.0),
        )
}

/// "/home   120.4 GiB / 476.9 GiB (25%)", the stacked bar, then each watched directory.
fn fs_card(fs: &amane_disk_usage::FileSystemUsage, dirs: &[(&amane_disk_usage::DirectoryUsage, amane::Color)]) -> amane::Column {
    let title = amane::Text::new(fs.mount_point().to_string_lossy())
        .size(22.0)
        .color(crate::theme::BACKGROUND);
    let usage = amane::Text::new(format!(
        "{} / {} ({}%)",
        human(fs.used()),
        human(fs.total()),
        percent(fs.used(), fs.total())
    ))
    .size(22.0)
    .weight(amane::Weight::Light)
    .color(crate::theme::BACKGROUND_HIGHLIGHT);

    let header = amane::Row::new(amane::children![title, usage])
        .width(amane::Parent)
        .gap(24.0)
        .align(amane::Align::Start);

    /* One segment per watched directory, then whatever else is used, the rest is the free track */
    let mut segments = Vec::with_capacity(dirs.len() + 1);
    let mut drawn = 0.0;
    for (dir, color) in dirs {
        let width = share(dir.usage(), fs.total(), BAR_WIDTH).min(BAR_WIDTH - drawn);
        drawn += width;
        segments.push(segment(width, amane::Fill::from(*color), 1.0));
    }
    let used_width = share(fs.used(), fs.total(), BAR_WIDTH);
    let other = (used_width - drawn).max(0.0);
    segments.push(segment(
        other,
        amane::Fill::from(amane::Color::from(crate::theme::BACKGROUND_HIGHLIGHT)),
        0.6,
    ));

    let mut children: Vec<Box<dyn amane::Widget>> = vec![Box::new(header), Box::new(bar(BAR_WIDTH, segments))];

    let dir_lines: Vec<Box<dyn amane::Widget>> = dirs
        .iter()
        .map(|(dir, color)| Box::new(dir_line(dir, *color, fs.total())) as Box<dyn amane::Widget>)
        .collect();
    if !dir_lines.is_empty() {
        children.push(Box::new(crate::utils::indent(
            DIR_INDENT,
            amane::Column::new(dir_lines).width(amane::Parent).gap(6.0),
        )));
    }

    amane::Column::new(children).width(amane::Parent).gap(4.0)
}

/// "name: 12.3 GiB" and a bar of the directory's size over the whole file system.
fn dir_line(dir: &amane_disk_usage::DirectoryUsage, color: amane::Color, fs_total: u64) -> amane::Column {
    let label = amane::Text::new(format!("{}: {}", dir.name(), human(dir.usage())))
        .size(18.0)
        .weight(amane::Weight::Light)
        .color(crate::theme::BACKGROUND);

    let width = BAR_WIDTH - DIR_INDENT;
    let fill = share(dir.usage(), fs_total, width);
    let bar = bar(width, vec![segment(fill, amane::Fill::from(color), 1.0)]);

    amane::Column::new(amane::children![label, bar]).width(amane::Parent).gap(2.0)
}

/// The free-space track, with the used segments laid left to right over it.
fn bar(width: f32, segments: Vec<Box<dyn amane::Widget>>) -> amane::Rectangle {
    amane::Rectangle::new()
        .width(width)
        .height(BAR_HEIGHT)
        .fill(TRACK)
        .radius(2.0)
        .align_child(amane::Align::Start, amane::Align::Start)
        .child(amane::Row::new(segments))
}

fn segment(width: f32, fill: amane::Fill, opacity: f32) -> Box<dyn amane::Widget> {
    Box::new(
        amane::Rectangle::new()
            .width(width.max(0.0))
            .height(BAR_HEIGHT)
            .fill(fill)
            .opacity(opacity),
    )
}

/// Pixels taken by `part` on a bar of `width` representing `total`.
fn share(part: u64, total: u64, width: f32) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (part as f64 / total as f64 * width as f64).min(width as f64) as f32
}

fn percent(part: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        (part as f64 / total as f64 * 100.0).round() as u64
    }
}

/// 1536 -> "1.5 KiB"
fn human(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
