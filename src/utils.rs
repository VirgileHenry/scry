/// Compute the text width to place it at the correct position
pub fn text_width(text: &str, size: f32) -> f32 {
    /* amane has no size-to-content yet, so widths are guessed from the text */
    /* Poppins averages about 0.62 of its size per character */
    text.chars().count() as f32 * size * 0.62
}

/// Creates a new label with the common styling
pub fn label(text: &str, size: f32) -> amane::Text {
    amane::Text::new(text)
        .size(size)
        .font(crate::theme::BODY)
        .weight(amane::Weight::SemiBold)
        .color(crate::theme::TEXT)
}

/// a rounded capsule of a given width
pub fn pill(width: impl Into<amane::Size>, height: impl Into<amane::Size>, fill: impl Into<amane::Fill>) -> amane::Rectangle {
    amane::Rectangle::new()
        .width(width)
        .height(height)
        .radius(amane::Full)
        .fill(fill)
}

/// a thin track with the value drawn over it, from the top clockwise
pub fn ring(size: f32, thickness: f32, value: f32, color: amane::Color, track: amane::Color) -> amane::Canvas {
    use amane::Shape;

    let middle = size / 2.0;
    let radius = middle - thickness / 2.0;
    let sweep = 360.0 * value.clamp(0.0, 1.0);

    amane::Canvas::new().width(size).height(size).shapes(amane::shapes![
        amane::Arc::new()
            .center(middle, middle)
            .radius(radius)
            .stroke(thickness, track),
        amane::Arc::new()
            .center(middle, middle)
            .radius(radius)
            .sweep(sweep)
            .stroke(thickness, color)
            .cap(amane::Cap::Round),
    ])
}

/// A glyph as an icon via text
pub fn icon(glyph: &str, size: f32, color: amane::Color) -> amane::Text {
    amane::Text::new(glyph).size(size).font(crate::theme::NERD).color(color)
}

/// Add ident by adding an empty rectanlge in a row in front of it
pub fn indent(ident: f32, widget: impl amane::Widget + 'static) -> amane::Row {
    let ident = amane::Rectangle::new().width(ident).height(0.0);
    amane::Row::new(amane::children![ident, widget]).width(amane::Parent)
}
