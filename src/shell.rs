use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct DesktopShell {
    _manager: Option<GlobalHotKeyManager>,
    shortcut_id: Option<u32>,
    _tray: Option<TrayIcon>,
}

impl DesktopShell {
    pub fn new() -> Self {
        let (manager, shortcut_id) = create_shortcut();
        Self {
            _manager: manager,
            shortcut_id,
            _tray: create_tray_icon(),
        }
    }

    pub fn force_full_toggle_requested(&self) -> bool {
        let Some(shortcut_id) = self.shortcut_id else {
            return false;
        };

        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == shortcut_id && event.state == HotKeyState::Pressed {
                return true;
            }
        }
        false
    }

    pub fn compact_toggle_requested(&self) -> bool {
        false
    }
}

fn create_shortcut() -> (Option<GlobalHotKeyManager>, Option<u32>) {
    let Ok(manager) = GlobalHotKeyManager::new() else {
        return (None, None);
    };
    let shortcut = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
    let id = shortcut.id();
    if manager.register(shortcut).is_err() {
        return (Some(manager), None);
    }
    (Some(manager), Some(id))
}

fn create_tray_icon() -> Option<TrayIcon> {
    let size = 16_u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let edge = x == 0 || y == 0 || x == size - 1 || y == size - 1;
            let value = if edge { 70 } else { 150 };
            rgba.extend_from_slice(&[value, value, value, 255]);
        }
    }

    let icon = Icon::from_rgba(rgba, size, size).ok()?;
    TrayIconBuilder::new()
        .with_tooltip("LexiPath")
        .with_icon(icon)
        .build()
        .ok()
}
