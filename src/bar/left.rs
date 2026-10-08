mod icons;
mod monitor_workspace;

use amane::Service;
use monitor_workspace::MonitorWorkspace;

/// Number of workspace this is meant for
const WORKSPACE_COUNT: usize = 10;

/// The size of the circle representing each worskpace.
/// Directly derived from the top bar item height.
const WORKSPACE_ITEM_SIZE: f32 = super::ITEM_HEIGHT;

/// Size of the icons as thumbnails
const ICON_THUMBNAIL_SIZE: u32 = 64;

/// How much space the desktop entry icons takes in the workspace item.
const WORKSPACE_ICON_SCALE: f32 = 0.65;

/// Service tracking the state of workspaces to display on the top bar
struct Workspaces {
    /// Collections and cache of icons per window class.
    icons: icons::IconCollection,
    /// For each workspace, the icon we want to display for that workspace.
    workspace_icons: [Option<icons::DesktopIcon>; WORKSPACE_COUNT],
    /// For each monitor, the workspace currently present on that monitor,
    /// as well as the transitions from other workspaces.
    monitor_workspaces: std::collections::HashMap<hipc::types::MonitorName, MonitorWorkspace>,
}

impl Workspaces {
    /// Synchronize the workspaces state with hyprland.
    fn sync_workspaces(&mut self, workspaces: &[hipc::types::Workspace]) {
        /* Reset the bar */
        self.workspace_icons.iter_mut().for_each(|icon| *icon = None);

        /* For each workspace, get the last active window to get the icon */
        for workspace in workspaces.iter() {
            let Some(icon) = self.icons.get(&workspace.last_window) else {
                continue;
            };
            let Ok(workspace_index) = usize::try_from(workspace.id.raw() - 1) else {
                continue;
            };
            let Some(workspace_icon) = self.workspace_icons.get_mut(workspace_index) else {
                continue;
            };
            *workspace_icon = Some(icon.clone());
        }
    }

    /// Synchronize the workspaces state with hyprland.
    fn sync_monitors(&mut self, monitors: &[hipc::types::Monitor]) {
        /* for each monitor, get which workspace is active on that monitor */
        for monitor in monitors.iter() {
            let entry = self.monitor_workspaces.entry(monitor.name.clone());
            let monitor_workspace = entry.or_insert_with(|| MonitorWorkspace::new(monitor.active_workspace.id));
            monitor_workspace.set_workspace(monitor.active_workspace.id);
        }
    }
}

impl Service for Workspaces {
    fn load() -> Self {
        Self {
            icons: icons::IconCollection::new(),
            workspace_icons: std::array::from_fn(|_| None),
            monitor_workspaces: std::collections::HashMap::new(),
        }
    }

    fn listen() {
        let mut socket = match hipc::HyprlandEventSocket::connect() {
            Ok(socket) => socket,
            Err(e) => {
                tracing::error!("Failed to connect to hyprland socket: {e}");
                panic!("Failed to connect to hyprland socket: {e}");
            }
        };

        match hipc::commands::clients() {
            Ok(clients) => Self::write().icons.sync(&clients),
            Err(e) => tracing::warn!("Failed to fetch clients: {e}"),
        };
        match hipc::commands::workspaces() {
            Ok(workspaces) => Self::write().sync_workspaces(&workspaces),
            Err(e) => tracing::warn!("Failed to fetch workspaces: {e}"),
        };
        match hipc::commands::monitors() {
            Ok(monitors) => Self::write().sync_monitors(&monitors),
            Err(e) => tracing::warn!("Failed to fetch monitors: {e}"),
        };

        let mut dirty_workspaces = false;
        let mut dirty_monitors = false;

        loop {
            match socket.read() {
                Ok(hipc::HyprlandEvent::OpenWindow { address, class, .. }) => {
                    Self::write().icons.insert(address, &class);
                    dirty_workspaces = true;
                }
                Ok(hipc::HyprlandEvent::CloseWindow { address }) => {
                    Self::write().icons.remove(&address);
                    dirty_workspaces = true;
                }
                Ok(
                    hipc::HyprlandEvent::MoveWindowV2 { .. }
                    | hipc::HyprlandEvent::ActiveWindowV2 { .. }
                    | hipc::HyprlandEvent::DestroyWorkspaceV2 { .. },
                ) => dirty_workspaces = true,
                Ok(
                    hipc::HyprlandEvent::WorkspaceV2 { .. }
                    | hipc::HyprlandEvent::FocusedMonitorV2 { .. }
                    | hipc::HyprlandEvent::MoveWorkspaceV2 { .. }
                    | hipc::HyprlandEvent::MonitorAddedV2 { .. }
                    | hipc::HyprlandEvent::MonitorRemovedV2 { .. },
                ) => dirty_monitors = true,
                Ok(_) => {}
                Err(e) => tracing::error!("Failed to get hyprland event: {e}"),
            }

            if !socket.is_empty() {
                continue; /* Avoid multiple hyprland commands on event bursts */
            }

            if dirty_workspaces {
                match hipc::commands::workspaces() {
                    Ok(workspaces) => Self::write().sync_workspaces(&workspaces),
                    Err(e) => tracing::warn!("Failed to fetch workspaces: {e}"),
                };
                dirty_workspaces = false;
            }
            if dirty_monitors {
                match hipc::commands::monitors() {
                    Ok(monitors) => Self::write().sync_monitors(&monitors),
                    Err(e) => tracing::warn!("Failed to fetch monitors: {e}"),
                };
                dirty_monitors = false;
            }
        }
    }
}

pub fn view(monitor: &amane::Monitor, width: f32) -> amane::Rectangle {
    amane::Rectangle::new()
        .width(width)
        .height(amane::Parent)
        .align_child(amane::Start, amane::Center)
        .padding(amane::Padding {
            left: super::HORIZONTAL_PADDING,
            ..amane::Padding::default()
        })
        .child(strip(monitor))
}

// ten slots, with the accent highlight sliding over the one this monitor shows
fn strip(monitor: &amane::Monitor) -> amane::Rectangle {
    let workspaces = Workspaces::read();

    let monitor_workspace = workspaces.monitor_workspaces.get(&monitor.name);

    let slots: Vec<Box<dyn amane::Widget>> = workspaces
        .workspace_icons
        .iter()
        .enumerate()
        .map(|(workspace_index, icon)| {
            let workspace_higlight_size = match monitor_workspace {
                Some(monitor_workspace) => WORKSPACE_ITEM_SIZE * monitor_workspace.anim_value(workspace_index),
                None => 0.0,
            };
            Box::new(
                workspace_slot().child(
                    amane::Rectangle::new()
                        .width(workspace_higlight_size)
                        .height(workspace_higlight_size)
                        .radius(amane::Full)
                        .fill(crate::theme::BORDER)
                        .align_child(amane::Center, amane::Center)
                        .child(match icon {
                            Some(icon) => icon.to_widget(),
                            None => workspace_text_widget("·"),
                        }),
                ),
            ) as Box<dyn amane::Widget>
        })
        .collect();

    const STRIP_WIDTH: f32 = WORKSPACE_ITEM_SIZE * WORKSPACE_COUNT as f32;
    const STRIP_HEIGHT: f32 = WORKSPACE_ITEM_SIZE;

    crate::utils::pill(STRIP_WIDTH, STRIP_HEIGHT, crate::theme::BACKGROUND_HIGHLIGHT)
        .align_child(amane::Center, amane::Center)
        .child(amane::Row::new(slots))
}

fn workspace_slot() -> amane::Rectangle {
    amane::Rectangle::new()
        .width(WORKSPACE_ITEM_SIZE)
        .height(WORKSPACE_ITEM_SIZE)
        .align_child(amane::Center, amane::Center)
}

fn workspace_text_widget(text: &str) -> amane::Rectangle {
    workspace_slot().child(
        amane::Text::new(text)
            .size(WORKSPACE_ITEM_SIZE * 0.6)
            .color(crate::theme::TEXT),
    )
}

fn workspace_icon_widget(icon_path: &std::path::PathBuf) -> amane::Rectangle {
    /* Images as thumbnail allows amane to not unload them */
    let image = amane::Image::contain(icon_path).thumbnail(ICON_THUMBNAIL_SIZE, ICON_THUMBNAIL_SIZE);
    workspace_slot().child(
        amane::Rectangle::new()
            .width(WORKSPACE_ITEM_SIZE * WORKSPACE_ICON_SCALE)
            .height(WORKSPACE_ITEM_SIZE * WORKSPACE_ICON_SCALE)
            .fill(image),
    )
}
