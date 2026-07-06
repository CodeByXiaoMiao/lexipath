use eframe::egui;

use crate::app_v2::LexiPathApp;
use crate::course::CoursePack;
use crate::fonts;
use crate::ipa_app::IpaApp;
use crate::progress_store::ProgressStore;

pub struct RootApp {
    ipa: Option<IpaApp>,
    vocabulary: LexiPathApp,
    root_status: String,
    allow_extra_new_units_today: bool,
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
            root_status: String::new(),
            allow_extra_new_units_today: false,
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
}

impl eframe::App for RootApp {
    fn update(&mut self, context: &egui::Context, frame: &mut eframe::Frame) {
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
