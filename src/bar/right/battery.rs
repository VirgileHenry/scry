use std::path::{Path, PathBuf};
use std::time::Duration;

use amane::Service;

const POWER_SUPPLIES: &str = "/sys/class/power_supply";

/// The laptop's own battery, ignoring wireless peripherals.
#[derive(Default)]
pub struct Battery {
    /// None on a machine without a system battery.
    path: Option<PathBuf>,
    /// 0 to 100.
    percent: u8,
    charging: bool,
    /// Plugged in and not charging, usually full or held at a charge threshold.
    full: bool,
}

impl Service for Battery {
    fn new() -> Self {
        let mut battery = Self {
            path: find(),
            ..Self::default()
        };
        battery.update();
        battery
    }

    fn interval() -> Duration {
        Duration::from_secs(5)
    }

    fn update(&mut self) -> bool {
        let Some(path) = &self.path else {
            return false;
        };

        let before = (self.percent, self.charging, self.full);
        let status = read(path, "status");

        self.percent = read(path, "capacity").parse().unwrap_or(0);
        self.charging = status == "Charging";
        self.full = status == "Full" || status == "Not charging";

        (self.percent, self.charging, self.full) != before
    }
}

impl Battery {
    pub fn percent(&self) -> u8 {
        self.percent
    }

    pub fn charging(&self) -> bool {
        self.charging
    }
}

/// The first battery that isn't a peripheral: mice and headsets report scope=Device.
fn find() -> Option<PathBuf> {
    std::fs::read_dir(POWER_SUPPLIES)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| read(path, "type") == "Battery" && read(path, "scope") != "Device")
}

fn read(folder: &Path, name: &str) -> String {
    std::fs::read_to_string(folder.join(name))
        .map(|text| text.trim().to_string())
        .unwrap_or_default()
}
