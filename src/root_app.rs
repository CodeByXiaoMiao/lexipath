use eframe::egui;

use crate::app_v2::LexiPathApp;
use crate::course::CoursePack;
use crate::fonts;
use crate::ipa_app::IpaApp;

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

        eframe::App::update(&mut self.vocabulary, context, frame);
    }
}
