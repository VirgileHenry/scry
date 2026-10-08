use amane::Service;

const SIZE: f32 = 18.0;

// the focused window's title, from hyprland's event socket
pub struct ActiveWindow {
    title: String,
}

impl Service for ActiveWindow {
    fn load() -> Self {
        let title = match hipc::commands::active_window() {
            Ok(window) => window.title.to_string(),
            Err(_) => ":)".to_string(),
        };
        Self { title }
    }

    // a panic here makes amane reconnect after 5 seconds
    fn listen() {
        let mut socket = match hipc::HyprlandEventSocket::connect() {
            Ok(socket) => socket,
            Err(e) => {
                tracing::error!("Failed to connect to hyprland socket: {e}");
                panic!("Failed to connect to hyprland socket: {e}");
            }
        };

        loop {
            match socket.read() {
                Ok(hipc::HyprlandEvent::ActiveWindow { title, .. }) => Self::write().title = title.to_string(),
                Ok(_) => {}
                Err(e) => tracing::error!("Failed to get hyprland event: {e}"),
            }
        }
    }
}

pub fn view(width: f32) -> amane::Rectangle {
    let window = ActiveWindow::read();

    /* long titles are cut off before they reach the sides */
    let text_width = crate::utils::text_width(&window.title, SIZE).min(width);

    amane::Rectangle::new()
        .width(width)
        .height(amane::Parent)
        .align_child(amane::Center, amane::Center)
        .child(
            amane::Rectangle::new()
                .width(text_width)
                .height(amane::Parent)
                .align_child(amane::Center, amane::Center)
                .child(crate::utils::label(&window.title, SIZE).elide()),
        )
}
