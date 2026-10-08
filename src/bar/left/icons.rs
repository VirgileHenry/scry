use freedesktop_desktop_entry as fde;

/// Collection of the current icons of all hyprland clients.
///
/// Also holds a cache and some nice helpers functions
pub struct IconCollection {
    /// Map of all active windows to their icons
    icons: std::collections::HashMap<hipc::types::WindowAddress, DesktopIcon>,
    /// Cache of the icons, mapping a given window class to the resolved icon path.
    icons_cache: std::collections::HashMap<hipc::types::WindowClass, DesktopIcon>,
    /// Desktop entries loaded for icon lookup
    desktop_entries: Vec<fde::DesktopEntry>,
}

impl IconCollection {
    /// Create a new empty icon collection.
    pub fn new() -> Self {
        /* fde loading thingies */
        let locales = fde::get_languages_from_env();
        let desktop_entries = fde::desktop_entries(&locales);

        Self {
            icons: std::collections::HashMap::new(),
            icons_cache: std::collections::HashMap::new(),
            desktop_entries,
        }
    }

    /// Sync the icons with the current client state, creating a mapping of each window to its icon.
    pub fn sync(&mut self, clients: &[hipc::types::Client]) {
        self.icons.clear();
        for client in clients.iter() {
            self.insert(client.address, &client.initial_class);
        }
    }

    /// Get one of the loaded icons
    pub fn get(&self, window_address: &hipc::types::WindowAddress) -> Option<&DesktopIcon> {
        self.icons.get(window_address)
    }

    pub fn insert(&mut self, window_address: hipc::types::WindowAddress, window_class: &hipc::types::WindowClass) {
        let icon = match self.icons_cache.get(window_class) {
            Some(icon) => icon.clone(),
            None => {
                let icon = icon_for_class(&self.desktop_entries, window_class);
                self.icons_cache.insert(window_class.clone(), icon.clone());
                icon
            }
        };
        self.icons.insert(window_address, icon);
    }

    pub fn remove(&mut self, window_address: &hipc::types::WindowAddress) {
        self.icons.remove(window_address);
    }
}

#[derive(Debug, Clone)]
pub struct DesktopIcon {
    path: Option<std::path::PathBuf>,
}

impl DesktopIcon {
    pub fn to_widget(&self) -> amane::Rectangle {
        match &self.path {
            Some(path) => super::workspace_icon_widget(path),
            None => super::workspace_text_widget("?"),
        }
    }
}

fn icon_for_class(entries: &[fde::DesktopEntry], window_class: &hipc::types::WindowClass) -> DesktopIcon {
    DesktopIcon {
        path: resolve_icon_entry(entries, window_class),
    }
}

fn resolve_icon_entry(entries: &[fde::DesktopEntry], class: &str) -> Option<std::path::PathBuf> {
    let icon_name = fde::find_app_by_id(entries, fde::unicase::Ascii::new(class))
        .and_then(|entry| entry.icon())
        .unwrap_or(class);

    if icon_name.starts_with('/') {
        Some(std::path::PathBuf::from(icon_name))
    } else {
        freedesktop_icons::lookup(icon_name)
            .with_theme(crate::theme::ICON_THEME)
            .with_size(48)
            .with_cache()
            .find()
    }
}
