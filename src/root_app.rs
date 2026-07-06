use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app_v2::LexiPathApp;
use crate::course::CoursePack;
use crate::fonts;
use crate::ipa_app::IpaApp;
use crate::progress_store::ProgressStore;
use crate::shell::DesktopShell;

const HIDDEN_HOVER_CHECK: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct UiSettings {
    enable_transparent_mode: bool,
    enable_hover_show_hide: bool,
    visible_opacity_percent: u8,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            enable_transparent_mode: false,
            enable_hover_show_hide: false,
            visible_opacity_percent: 85,
        }
    }
}

impl UiSettings {
    fn load() -> Self {
        let Ok(path) = settings_path() else {
            return Self::default();
        };
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut settings = serde_json::from_str::<Self>(&text).unwrap_or_default();
        settings.normalize();
        settings
    }

    fn save(&mut self) -> anyhow::Result<()> {
        self.normalize();
        let path = settings_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        fs::rename(temporary, path)?;
        Ok(())
    }

    fn visible_alpha(&self) -> u8 {
        ((u16::from(self.visible_opacity_percent) * 255) / 100) as u8
    }

    fn normalize(&mut self) {
        self.visible_opacity_percent = self.visible_opacity_percent.clamp(20, 100);
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    let executable = std::env::current_exe().context("failed to locate executable")?;
    Ok(executable
        .parent()
        .context("executable has no parent directory")?
        .join("data")
        .join("settings.json"))
}

#[cfg(target_os = "windows")]
struct TransparencyController {
    title: Vec<u16>,
    hwnd: windows_sys::Win32::Foundation::HWND,
    force_full: bool,
    hidden: bool,
    hidden_rect: Option<windows_sys::Win32::Foundation::RECT>,
    last_alpha: Option<u8>,
    click_through: bool,
    pointer_initialized: bool,
    pointer_was_inside: bool,
    last_hidden_check: Instant,
}

#[cfg(target_os = "windows")]
impl TransparencyController {
    fn new(window_title: &str) -> Self {
        let mut title = window_title.encode_utf16().collect::<Vec<_>>();
        title.push(0);
        Self {
            title,
            hwnd: 0 as windows_sys::Win32::Foundation::HWND,
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

    fn toggle_force_full(&mut self) {
        self.force_full = !self.force_full;
        if self.force_full {
            self.hidden = false;
            self.set_click_through(false);
            self.set_alpha(255);
        } else {
            self.last_alpha = None;
        }
    }

    fn force_full(&self) -> bool {
        self.force_full
    }

    fn hidden(&self) -> bool {
        self.hidden
    }

    fn update(&mut self, context: &egui::Context, settings: &UiSettings, pointer_inside_viewport: bool) {
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

    fn hwnd(&mut self) -> windows_sys::Win32::Foundation::HWND {
        if self.hwnd.is_null() {
            self.hwnd = unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                    std::ptr::null(),
                    self.title.as_ptr(),
                )
            };
        }
        self.hwnd
    }

    fn capture_rect(&mut self) {
        let hwnd = self.hwnd();
        if hwnd.is_null() {
            return;
        }
        let mut rect = windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect) } != 0 {
            self.hidden_rect = Some(rect);
        }
    }

    fn cursor_inside_hidden_rect(&self) -> bool {
        let Some(rect) = self.hidden_rect else {
            return false;
        };
        let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        if unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) } == 0 {
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
            windows_sys::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes(
                hwnd,
                0,
                alpha,
                windows_sys::Win32::UI::WindowsAndMessaging::LWA_ALPHA,
            );
        }
        self.last_alpha = Some(alpha);
    }

    fn ensure_layered(&mut self, enable: bool) {
        let hwnd = self.hwnd();
        if hwnd.is_null() {
            return;
        }
        unsafe {
            let style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
            );
            let mut wanted = style;
            if enable {
                wanted |= windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED as isize;
            } else {
                wanted &= !(windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED as isize);
            }
            if wanted != style {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
                    wanted,
                );
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    0 as windows_sys::Win32::Foundation::HWND,
                    0,
                    0,
                    0,
                    0,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED,
                );
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
            let style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
            );
            let mut wanted = style;
            if enable {
                wanted |= windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED as isize;
                wanted |= windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT as isize;
            } else {
                wanted &= !(windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT as isize);
            }
            if wanted != style {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
                    wanted,
                );
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    0 as windows_sys::Win32::Foundation::HWND,
                    0,
                    0,
                    0,
                    0,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED,
                );
            }
        }
        self.click_through = enable;
    }
}

#[cfg(not(target_os = "windows"))]
struct TransparencyController {
    force_full: bool,
}

#[cfg(not(target_os = "windows"))]
impl TransparencyController {
    fn new(_window_title: &str) -> Self {
        Self { force_full: false }
    }

    fn toggle_force_full(&mut self) {
        self.force_full = !self.force_full;
    }

    fn force_full(&self) -> bool {
        self.force_full
    }

    fn hidden(&self) -> bool {
        false
    }

    fn update(&mut self, _context: &egui::Context, _settings: &UiSettings, _pointer_inside_viewport: bool) {}
}

pub struct RootApp {
    ipa: Option<IpaApp>,
    vocabulary: LexiPathApp,
    settings: UiSettings,
    shell: DesktopShell,
    transparency: TransparencyController,
    root_status: String,
    allow_extra_new_units_today: bool,
    show_window_settings: bool,
}

impl RootApp {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        course: CoursePack,
    ) -> anyhow::Result<Self> {
        fonts::install(&context.egui_ctx);
        Ok(Self {
            ipa: IpaApp::load()?,
            vocabulary: LexiPathApp::new(context, course),
            settings: UiSettings::load(),
            shell: DesktopShell::new(),
            transparency: TransparencyController::new("Reference"),
            root_status: String::new(),
            allow_extra_new_units_today: false,
            show_window_settings: false,
        })
    }

    fn vocabulary_locked_today(&self) -> bool {
        if self.allow_extra_new_units_today {
            return false;
        }
        ProgressStore::open()
            .map(|store| store.vocabulary_locked_today())
            .unwrap_or(false)
    }

    fn show_daily_complete(&mut self, context: &egui::Context) {
        egui::TopBottomPanel::top("daily_header").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.strong("LexiPath");
                ui.separator();
                ui.label("固定学习计划");
            });
        });
        egui::CentralPanel::default().show(context, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("今日新课已完成");
                ui.label("今天已经完成两个 6 词单元，共 12 个新词。");
                ui.label("到期复习仍会优先开放；需要继续学习时，可以手动进入下一天。");
                if ui.button("进入下一天").clicked() {
                    self.allow_extra_new_units_today = true;
                    self.root_status = "已进入下一天：本次运行会继续开放后续新课；到期复习仍会优先。".to_owned();
                    context.request_repaint();
                }
                if !self.root_status.is_empty() {
                    ui.label(&self.root_status);
                }
            });
        });
    }

    fn show_window_settings(&mut self, context: &egui::Context) {
        let mut changed = false;
        egui::TopBottomPanel::top("window_settings").show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("LexiPath");
                ui.separator();
                if ui.small_button(if self.show_window_settings { "收起窗口设置" } else { "窗口设置" }).clicked() {
                    self.show_window_settings = !self.show_window_settings;
                }
                if self.transparency.force_full() {
                    ui.separator();
                    ui.label("Ctrl + Alt + Space：当前临时 100% 显示，再按一次恢复。");
                } else if self.transparency.hidden() {
                    ui.separator();
                    ui.label("窗口已隐藏；鼠标回到原窗口区域会恢复。");
                }
            });
            if self.show_window_settings {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    changed |= ui.checkbox(&mut self.settings.enable_transparent_mode, "启用透明模式").changed();
                    changed |= ui
                        .add_enabled(
                            self.settings.enable_transparent_mode,
                            egui::Checkbox::new(&mut self.settings.enable_hover_show_hide, "启用鼠标进入显示、离开隐藏"),
                        )
                        .changed();
                    ui.label("显示时透明度");
                    let mut opacity = u32::from(self.settings.visible_opacity_percent);
                    if ui
                        .add_enabled(
                            self.settings.enable_transparent_mode,
                            egui::Slider::new(&mut opacity, 20..=100).suffix("%"),
                        )
                        .changed()
                    {
                        self.settings.visible_opacity_percent = opacity as u8;
                        changed = true;
                    }
                });
            }
        });
        if changed {
            if let Err(error) = self.settings.save() {
                self.root_status = format!("保存设置失败：{error}");
            }
        }
    }

    fn update_transparency(&mut self, context: &egui::Context) {
        if self.shell.force_full_toggle_requested() {
            self.transparency.toggle_force_full();
            context.request_repaint();
        }
        let pointer_inside_viewport = context.input(|input| input.pointer.hover_pos().is_some());
        self.transparency.update(context, &self.settings, pointer_inside_viewport);
    }
}

impl eframe::App for RootApp {
    fn update(&mut self, context: &egui::Context, frame: &mut eframe::Frame) {
        self.update_transparency(context);
        self.show_window_settings(context);

        if let Some(ipa) = &mut self.ipa {
            if ipa.update(context) {
                self.ipa = None;
                context.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    620.0, 520.0,
                )));
            }
            return;
        }

        if self.vocabulary_locked_today() {
            self.show_daily_complete(context);
            return;
        }

        eframe::App::update(&mut self.vocabulary, context, frame);
    }
}
