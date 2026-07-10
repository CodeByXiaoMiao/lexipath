use eframe::egui;

use crate::app_v2::LexiPathApp;
use crate::course::CoursePack;
use crate::fonts;
use crate::ipa_app::IpaApp;
use crate::progress_store::ProgressStore;

pub struct RootApp {
    ipa: Option<IpaApp>,
    vocabulary: LexiPathApp,
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
        })
    }

    fn vocabulary_locked_today(&self) -> bool {
        ProgressStore::open()
            .map(|store| store.vocabulary_locked_today())
            .unwrap_or(false)
    }

    fn show_daily_complete(&self, context: &egui::Context) {
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
                ui.label("新的词汇单元将在明天开放；到期复习仍会优先开放。");
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
