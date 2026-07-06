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

pub struct RootApp {
    ipa: Option<IpaApp>,
    vocabulary: LexiPathApp,
    root_status: String,
    allow_extra_new_units_today: bool,
    settings: UiSettings,
    show_window_settings: bool,
    pointer_faded: bool,
}

impl RootApp {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        course: CoursePack,
    ) -> anyhow::Result<Self> {
        fonts::install(&context.egui_ctx);
        let settings = UiSettings::load();
        apply_soft_transparency(&context.egui_ctx, &settings, false);
        Ok(Self {
            ipa: IpaApp::load()?,
            vocabulary: LexiPathApp::new(context, course),
            root_status: String::new(),
            allow_extra_new_units_today: false,
            settings,
            show_window_settings: false,
            pointer_faded: false,
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
                if ui
                    .small_button(if self.show_window_settings { "收起窗口设置" } else { "窗口设置" })
                    .clicked()
                {
                    self.show_window_settings = !self.show_window_settings;
                }
                if self.pointer_faded {
                    ui.separator();
                    ui.label("鼠标已移出：界面已降到 0%。");
                }
            });
            if self.show_window_settings {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    changed |= ui
                        .checkbox(&mut self.settings.enable_soft_transparency, "弱化透明模式")
                        .changed();
                    changed |= ui
                        .checkbox(&mut self.settings.enable_hover_fade, "鼠标移出淡隐 / 移入显示")
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.settings.visible_opacity_percent, 5..=100)
                                .text("界面透明度 %"),
                        )
                        .changed();
                });
                ui.label("弱化版：鼠标移出后把界面降到 0%，鼠标移入后恢复到滑块透明度；不做点击穿透。");
            }
        });
        if changed {
            apply_soft_transparency(context, &self.settings, self.pointer_faded);
            if let Err(error) = self.settings.save() {
                self.root_status = format!("窗口设置保存失败：{error:#}");
            }
        }
    }

    fn update_pointer_fade(&mut self, context: &egui::Context) {
        let faded = self.settings.enable_soft_transparency
            && self.settings.enable_hover_fade
            && context.input(|input| input.pointer.hover_pos().is_none());
        if faded != self.pointer_faded {
            self.pointer_faded = faded;
            apply_soft_transparency(context, &self.settings, self.pointer_faded);
        }
        if self.settings.enable_soft_transparency && self.settings.enable_hover_fade {
            context.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }
}

impl eframe::App for RootApp {
    fn update(&mut self, context: &egui::Context, frame: &mut eframe::Frame) {
        self.update_pointer_fade(context);
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

fn apply_soft_transparency(context: &egui::Context, settings: &UiSettings, faded: bool) {
    let mut style = (*context.style()).clone();
    let alpha = if settings.enable_soft_transparency {
        if faded {
            0
        } else {
            settings.alpha()
        }
    } else {
        255
    };

    style.visuals.panel_fill = style.visuals.panel_fill.linear_multiply(f32::from(alpha) / 255.0);
    style.visuals.window_fill = style.visuals.window_fill.linear_multiply(f32::from(alpha) / 255.0);
    style.visuals.extreme_bg_color = style
        .visuals
        .extreme_bg_color
        .linear_multiply(f32::from(alpha) / 255.0);
    context.set_style(style);
}
