use std::time::Duration;

use eframe::egui;

use crate::settings::UiSettings;

const HIDDEN_HOVER_CHECK: Duration = Duration::from_millis(150);

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr::null;
    use std::time::Instant;

    use super::{egui, UiSettings, HIDDEN_HOVER_CHECK};
    use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetCursorPos, GetWindowLongPtrW, GetWindowRect, SetLayeredWindowAttributes,
        SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOMOVE,
        SWP_NOSIZE, SWP_NOZORDER, WS_EX_LAYERED, WS_EX_TRANSPARENT,
    };

    pub struct TransparencyController {
        title: Vec<u16>,
        hwnd: HWND,
        force_full: bool,
        hidden: bool,
        hidden_rect: Option<RECT>,
        last_alpha: Option<u8>,
        click_through: bool,
        pointer_initialized: bool,
        pointer_was_inside: bool,
        last_hidden_check: Instant,
    }

    impl TransparencyController {
        pub fn new(window_title: &str) -> Self {
            let mut title = window_title.encode_utf16().collect::<Vec<_>>();
            title.push(0);
            Self {
                title,
                hwnd: 0 as HWND,
                force_full: false,
                hidden: false,
                hidden_rect: None,
                last_alpha: None,
                click_through: false,
                pointer_initialized: false,
                pointer_was_inside: false,
                last_hidden_check: Instant::now(),
            }
        }

        pub fn toggle_force_full(&mut self) {
            self.force_full = !self.force_full;
            if self.force_full {
                self.hidden = false;
                self.set_click_through(false);
                self.set_alpha(255);
            } else {
                self.last_alpha = None;
            }
        }

        pub fn force_full(&self) -> bool {
            self.force_full
        }

        pub fn hidden(&self) -> bool {
            self.hidden
        }

        pub fn update(&mut self, context: &egui::Context, settings: &UiSettings, pointer_inside_viewport: bool) {
            if self.hwnd().is_null() {
                return;
            }

            if !settings.enable_transparent_mode {
                self.hidden = false;
                self.set_click_through(false);
                self.set_alpha(255);
                self.pointer_initialized = false;
                return;
            }

            if self.force_full {
                self.hidden = false;
                self.set_click_through(false);
                self.set_alpha(255);
                return;
            }

            if !settings.enable_hover_show_hide {
                self.hidden = false;
                self.set_click_through(false);
                self.set_alpha(settings.visible_alpha());
                self.pointer_initialized = false;
                return;
            }

            if self.hidden {
                context.request_repaint_after(HIDDEN_HOVER_CHECK);
                if self.last_hidden_check.elapsed() >= HIDDEN_HOVER_CHECK {
                    self.last_hidden_check = Instant::now();
                    if self.cursor_inside_hidden_rect() {
                        self.hidden = false;
                        self.pointer_initialized = false;
                        self.set_click_through(false);
                        self.set_alpha(settings.visible_alpha());
                    }
                }
                return;
            }

            if !self.pointer_initialized {
                self.pointer_initialized = true;
                self.pointer_was_inside = pointer_inside_viewport;
            }

            if self.pointer_was_inside && !pointer_inside_viewport {
                self.capture_rect();
                self.hidden = true;
                self.last_hidden_check = Instant::now();
                self.set_click_through(true);
                self.set_alpha(0);
                context.request_repaint_after(HIDDEN_HOVER_CHECK);
            } else {
                self.set_click_through(false);
                self.set_alpha(settings.visible_alpha());
            }

            self.pointer_was_inside = pointer_inside_viewport;
        }

        fn hwnd(&mut self) -> HWND {
            if self.hwnd.is_null() {
                self.hwnd = unsafe { FindWindowW(null(), self.title.as_ptr()) };
            }
            self.hwnd
        }

        fn capture_rect(&mut self) {
            let hwnd = self.hwnd();
            if hwnd.is_null() {
                return;
            }
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            if unsafe { GetWindowRect(hwnd, &mut rect) } != 0 {
                self.hidden_rect = Some(rect);
            }
        }

        fn cursor_inside_hidden_rect(&self) -> bool {
            let Some(rect) = self.hidden_rect else {
                return false;
            };
            let mut point = POINT { x: 0, y: 0 };
            if unsafe { GetCursorPos(&mut point) } == 0 {
                return false;
            }
            point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
        }

        fn set_alpha(&mut self, alpha: u8) {
            if self.last_alpha == Some(alpha) {
                return;
            }
            let hwnd = self.hwnd();
            if hwnd.is_null() {
                return;
            }
            self.ensure_layered(alpha != 255);
            unsafe {
                SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
            }
            self.last_alpha = Some(alpha);
        }

        fn ensure_layered(&mut self, enable: bool) {
            let hwnd = self.hwnd();
            if hwnd.is_null() {
                return;
            }
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let mut wanted = style;
                if enable {
                    wanted |= WS_EX_LAYERED as isize;
                } else {
                    wanted &= !(WS_EX_LAYERED as isize);
                }
                if wanted != style {
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, wanted);
                    SetWindowPos(hwnd, 0 as HWND, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                }
            }
        }

        fn set_click_through(&mut self, enable: bool) {
            if self.click_through == enable {
                return;
            }
            let hwnd = self.hwnd();
            if hwnd.is_null() {
                return;
            }
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let mut wanted = style;
                if enable {
                    wanted |= WS_EX_LAYERED as isize;
                    wanted |= WS_EX_TRANSPARENT as isize;
                } else {
                    wanted &= !(WS_EX_TRANSPARENT as isize);
                }
                if wanted != style {
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, wanted);
                    SetWindowPos(hwnd, 0 as HWND, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                }
            }
            self.click_through = enable;
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::{egui, UiSettings};

    pub struct TransparencyController {
        force_full: bool,
    }

    impl TransparencyController {
        pub fn new(_window_title: &str) -> Self {
            Self { force_full: false }
        }

        pub fn toggle_force_full(&mut self) {
            self.force_full = !self.force_full;
        }

        pub fn force_full(&self) -> bool {
            self.force_full
        }

        pub fn hidden(&self) -> bool {
            false
        }

        pub fn update(&mut self, _context: &egui::Context, _settings: &UiSettings, _pointer_inside_viewport: bool) {}
    }
}

pub use platform::TransparencyController;
