use std::fs;
use std::path::PathBuf;

use anyhow::Context as _;
use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app_v2::LexiPathApp;
use crate::course::CoursePack;
use crate::fonts;
use crate::ipa_app::IpaApp;
use crate::progress_store::ProgressStore;

const WINDOW_TITLE: &str = "LexiPath";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct UiSettings {
    enable_soft_transparency: bool,
    enable_hover_fade: bool,
    visible_opacity_percent: u8,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            enable_soft_transparency: false,
            enable_hover_fade: false,
            visible_opacity_percent: 90,
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
        fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    fn alpha(&self) -> u8 {
        ((u16::from(self.visible_opacity_percent) * 255) / 100) as u8
    }

    fn normalize(&mut self) {
        self.visible_opacity_percent = self.visible_opacity_percent.clamp(5, 100);
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
struct NativeOpacity {
    title: Vec<u16>,
    hwnd: windows_sys::Win32::Foundation::HWND,
    current_alpha: Option<u8>,
}

#[cfg(target_os = "windows")]
impl NativeOpacity {
    fn new(title: &str) -> Self {
        let mut title = title.encode_utf16().collect::<Vec<_>>();
        title.push(0);
        Self {
            title,
            hwnd: std::ptr::null_mut(),
            current_alpha: None,
        }
    }

    fn set_alpha(&mut self, alpha: u8) {
        if self.current_alpha == Some(alpha) {
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
            let wanted = style | windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED as isize;
            if wanted != style {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
                    wanted,
                );
            }
            windows_sys::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes(
                hwnd,
                0,
                alpha,
                windows_sys::Win32::UI::WindowsAndMessaging::LWA_ALPHA,
            );
        }
        self.current_alpha = Some(alpha);
    }

    fn cursor_inside_window(&mut self) -> Option<bool> {
        let hwnd = self.hwnd();
        if hwnd.is_null() {
            return None;
        }
        let mut rect = windows_sys::Win32::Foundation::RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        let has_rect = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect) } != 0;
        let has_point = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) } != 0;
        if !has_rect || !has_point {
            return None;
        }
        Some(point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom)
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
}

#[cfg(not(target_os = "windows"))]
struct NativeOpacity;

#[cfg(not(target_os = "windows"))]
impl NativeOpacity {
    fn new(_title: &str) -> Self {
        Self
    }

    fn set_alpha(&mut self, _alpha: u8) {}

    fn cursor_inside_window(&mut self) -> Option<bool> {
        None
    }
}

pub struct RootApp {
    ipa: Option<IpaApp>,
    vocabulary: LexiPathApp,
    root_status: String,
    allow_extra_new_units_today: bool,
    settings: UiSettings,
    show_window_settings: bool,
    show_progress_settings: bool,
    ipa_progress_day_number: usize,
    vocabulary_progress_lesson_number: usize,
    pointer_faded: bool,
    topmost_applied: bool,
    opacity: NativeOpacity,
    theme_ready: bool,
}

impl RootApp {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        course: CoursePack,
    ) -> anyhow::Result<Self> {
        fonts::install(&context.egui_ctx);
        let settings = UiSettings::load();
        reset_normal_style(&context.egui_ctx);
        let ipa = IpaApp::load()?;
        let ipa_progress_day_number = ipa
            .as_ref()
            .map(IpaApp::current_day_number)
            .unwrap_or_else(|| IpaApp::total_day_count().max(1));
        let vocabulary = LexiPathApp::new(context, course);
        let vocabulary_progress_lesson_number = vocabulary.current_lesson_number();
        Ok(Self {
            ipa,
            vocabulary,
            root_status: String::new(),
            allow_extra_new_units_today: false,
            settings,
            show_window_settings: false,
            show_progress_settings: false,
            ipa_progress_day_number,
            vocabulary_progress_lesson_number,
            pointer_faded: false,
            topmost_applied: false,
            opacity: NativeOpacity::new(WINDOW_TITLE),
            theme_ready: false,
        })
    }

    fn active_progress_summary(&self) -> String {
        if let Some(ipa) = &self.ipa {
            format!("音标 · 第 {} / {} 天", ipa.current_day_number(), IpaApp::total_day_count())
        } else {
            format!("词汇 · {}", self.vocabulary.current_lesson_label())
        }
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
                ui.label("到期复习仍会优先开放；可以手动进入下一天，也可以任意切换课程进度。");
                ui.separator();
                self.show_progress_controls(ui, context);
                if !self.root_status.is_empty() {
                    ui.separator();
                    ui.label(&self.root_status);
                }
            });
        });
    }

    fn show_window_settings(&mut self, context: &egui::Context) {
        let mut changed = false;
        egui::TopBottomPanel::top("window_settings")
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(30, 41, 59))
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("LEXIPATH")
                        .strong()
                        .color(egui::Color32::from_rgb(45, 212, 191)),
                );
                ui.separator();
                if ui
                    .small_button(if self.show_window_settings { "窗口" } else { "窗口" })
                    .clicked()
                {
                    self.show_window_settings = !self.show_window_settings;
                }
                if ui
                    .small_button("进度")
                    .clicked()
                {
                    self.show_progress_settings = !self.show_progress_settings;
                    self.refresh_progress_inputs();
                }
                ui.separator();
                ui.label(
                    egui::RichText::new("窗口保持置顶")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(148, 163, 184)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(self.active_progress_summary())
                            .size(13.0)
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                });
                if self.pointer_faded {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("窗口已隐藏")
                            .size(13.0)
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                }
            });
            if self.show_window_settings {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    changed |= ui
                        .checkbox(&mut self.settings.enable_soft_transparency, "原生透明模式")
                        .changed();
                    changed |= ui
                        .checkbox(&mut self.settings.enable_hover_fade, "鼠标移出透明 / 移入显示")
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.settings.visible_opacity_percent, 5..=100)
                                .text("窗口透明度 %"),
                        )
                        .changed();
                });
                ui.label("使用 Windows 原生窗口透明度：鼠标移出降到 0%，鼠标回到窗口原区域后恢复；不点击穿透。 ");
            }
            if self.show_progress_settings {
                ui.separator();
                self.show_progress_controls(ui, context);
            }
        });
        if changed {
            self.apply_window_alpha();
            if let Err(error) = self.settings.save() {
                self.root_status = format!("窗口设置保存失败：{error:#}");
            }
        }
    }

    fn refresh_progress_inputs(&mut self) {
        self.ipa_progress_day_number = self
            .ipa
            .as_ref()
            .map(IpaApp::current_day_number)
            .unwrap_or(self.ipa_progress_day_number)
            .clamp(1, IpaApp::total_day_count().max(1));
        self.vocabulary_progress_lesson_number = self.vocabulary.current_lesson_number();
    }

    fn show_progress_controls(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        self.show_ipa_progress_controls(ui, context);
        ui.separator();
        self.show_vocabulary_progress_controls(ui, context);
    }

    fn show_ipa_progress_controls(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        let total = IpaApp::total_day_count().max(1);
        self.ipa_progress_day_number = self.ipa_progress_day_number.clamp(1, total);
        ui.strong("音标进度");
        ui.label(
            self.ipa
                .as_ref()
                .map(IpaApp::current_label)
                .unwrap_or_else(|| "当前音标：已完成或未打开音标模块".to_owned()),
        );
        ui.horizontal_wrapped(|ui| {
            if ui.button("进入下一天音标").clicked() {
                let result = if let Some(ipa) = self.ipa.as_mut() {
                    ipa.continue_after_daily_limit();
                    Ok(ipa.current_label())
                } else {
                    self.activate_ipa_day(self.ipa_progress_day_number)
                };
                self.apply_ipa_change(result, context);
            }
            if ui.button("上一天音标").clicked() {
                let target = self.ipa_progress_day_number.saturating_sub(1).clamp(1, total);
                self.ipa_progress_day_number = target;
                let result = self.activate_ipa_day(target);
                self.apply_ipa_change(result, context);
            }
            if ui.button("下一天音标").clicked() {
                let target = self.ipa_progress_day_number.saturating_add(1).clamp(1, total);
                self.ipa_progress_day_number = target;
                let result = self.activate_ipa_day(target);
                self.apply_ipa_change(result, context);
            }
            ui.label("指定第");
            ui.add(
                egui::DragValue::new(&mut self.ipa_progress_day_number)
                    .range(1..=total)
                    .speed(1.0),
            );
            ui.label(format!("/ {total} 天"));
            if ui.button("跳转音标").clicked() {
                let result = self.activate_ipa_day(self.ipa_progress_day_number);
                self.apply_ipa_change(result, context);
            }
        });
        ui.label("音标跳转只管理 14 天音标模块，不会改变词汇课进度。 ");
    }

    fn show_vocabulary_progress_controls(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        let total = self.vocabulary.lesson_count().max(1);
        self.vocabulary_progress_lesson_number = self.vocabulary_progress_lesson_number.clamp(1, total);
        ui.strong("词汇进度");
        ui.label(format!("当前词汇课：{}", self.vocabulary.current_lesson_label()));
        ui.horizontal_wrapped(|ui| {
            if ui.button("进入下一天词汇 / 继续后续新课").clicked() {
                self.allow_extra_new_units_today = true;
                self.vocabulary.continue_after_daily_limit();
                self.vocabulary_progress_lesson_number = self.vocabulary.current_lesson_number();
                self.root_status = "已手动进入下一天词汇/后续新课。".to_owned();
                context.request_repaint();
            }
            if ui.button("上一课").clicked() {
                let result = self.vocabulary.jump_relative_lesson(-1);
                self.apply_vocabulary_change(result, context);
            }
            if ui.button("下一课").clicked() {
                let result = self.vocabulary.jump_relative_lesson(1);
                self.apply_vocabulary_change(result, context);
            }
            ui.label("指定第");
            ui.add(
                egui::DragValue::new(&mut self.vocabulary_progress_lesson_number)
                    .range(1..=total)
                    .speed(1.0),
            );
            ui.label(format!("/ {total} 课"));
            if ui.button("跳转词汇课").clicked() {
                let result = self
                    .vocabulary
                    .jump_to_lesson_number(self.vocabulary_progress_lesson_number);
                self.apply_vocabulary_change(result, context);
            }
        });
        ui.label("词汇跳转只管理词汇课程；跳转词汇课会关闭音标页并保存到 data/progress.json。 ");
    }

    fn activate_ipa_day(&mut self, day_number: usize) -> Result<String, String> {
        if let Some(ipa) = self.ipa.as_mut() {
            ipa.jump_to_day_number(day_number)
        } else {
            match IpaApp::load_at_day_number(day_number) {
                Ok(ipa) => {
                    let message = ipa.current_label();
                    self.ipa_progress_day_number = ipa.current_day_number();
                    self.ipa = Some(ipa);
                    Ok(message)
                }
                Err(error) => Err(format!("切换音标进度失败：{error}")),
            }
        }
    }

    fn apply_ipa_change(&mut self, result: Result<String, String>, context: &egui::Context) {
        self.ipa_progress_day_number = self
            .ipa
            .as_ref()
            .map(IpaApp::current_day_number)
            .unwrap_or(self.ipa_progress_day_number);
        self.root_status = match result {
            Ok(message) => message,
            Err(error) => error,
        };
        context.request_repaint();
    }

    fn apply_vocabulary_change(&mut self, result: Result<String, String>, context: &egui::Context) {
        let success = result.is_ok();
        self.allow_extra_new_units_today = true;
        self.vocabulary_progress_lesson_number = self.vocabulary.current_lesson_number();
        self.root_status = match result {
            Ok(message) => message,
            Err(error) => error,
        };
        if success {
            self.ipa = None;
        }
        context.request_repaint();
    }

    fn update_pointer_fade(&mut self, context: &egui::Context) {
        let cursor_inside = self
            .opacity
            .cursor_inside_window()
            .unwrap_or_else(|| context.input(|input| input.pointer.hover_pos().is_some()));
        let faded = self.settings.enable_soft_transparency
            && self.settings.enable_hover_fade
            && !cursor_inside;
        if faded != self.pointer_faded {
            self.pointer_faded = faded;
            self.apply_window_alpha();
        }
        if self.settings.enable_soft_transparency && self.settings.enable_hover_fade {
            context.request_repaint_after(std::time::Duration::from_millis(120));
        }
    }

    fn apply_window_alpha(&mut self) {
        let alpha = if self.settings.enable_soft_transparency {
            if self.pointer_faded {
                0
            } else {
                self.settings.alpha()
            }
        } else {
            255
        };
        self.opacity.set_alpha(alpha);
    }

    fn ensure_topmost(&mut self, context: &egui::Context) {
        if self.topmost_applied {
            return;
        }
        context.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));
        self.topmost_applied = true;
    }
}

impl eframe::App for RootApp {
    fn update(&mut self, context: &egui::Context, frame: &mut eframe::Frame) {
        if !self.theme_ready {
            reset_normal_style(context);
            self.theme_ready = true;
        }
        self.ensure_topmost(context);
        self.apply_window_alpha();
        self.update_pointer_fade(context);
        self.show_window_settings(context);

        if let Some(ipa) = &mut self.ipa {
            if ipa.update(context) {
                self.ipa = None;
                context.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    900.0, 680.0,
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

fn reset_normal_style(context: &egui::Context) {
    let mut style = egui::Style::default();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_fill = egui::Color32::from_rgb(15, 23, 42);
    style.visuals.panel_fill = egui::Color32::from_rgb(15, 23, 42);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(30, 41, 59);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(15, 23, 42);
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(241, 245, 249));
    style.visuals.hyperlink_color = egui::Color32::from_rgb(45, 212, 191);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(51, 65, 85);
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(30, 41, 59);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(71, 85, 105);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(20, 184, 166);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(10);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(10);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(10);
    style.spacing.item_spacing = egui::vec2(10.0, 9.0);
    style.spacing.window_margin = egui::Margin::symmetric(20, 16);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.interact_size = egui::vec2(40.0, 36.0);
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(15.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(14.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(24.0),
    );
    context.set_style(style);
}
