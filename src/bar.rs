mod center;
mod left;
mod right;

pub const BAR_HEIGHT: f32 = 40.0;
pub const ITEM_HEIGHT: f32 = 32.0;
pub const HORIZONTAL_PADDING: f32 = 6.0;

pub fn view(monitor: &amane::Monitor) -> amane::LayerWindow {
    /* three equal thirds, so the title stays centered whatever the sides hold */
    let third = monitor.width as f32 / 3.0;

    let top_bar_content = amane::Rectangle::new()
        .width(amane::Parent)
        .height(BAR_HEIGHT)
        .child(amane::Row::new(amane::children![
            left::view(monitor, third),
            center::view(third),
            right::view(third),
        ]));
    let top_bar = amane::Rectangle::new()
        .width(amane::Parent)
        .height(amane::Parent)
        .fill(crate::theme::BACKGROUND)
        .border(1.0, crate::theme::BORDER)
        .clip()
        .child(top_bar_content);

    amane::LayerWindow::new()
        .width(amane::Full)
        .height(BAR_HEIGHT)
        .anchor_vertical(amane::Vertical::Top)
        .layer(amane::Layer::Top)
        .space(amane::Zone::Reserve)
        .namespace("amane-bar")
        .child(top_bar)
}
