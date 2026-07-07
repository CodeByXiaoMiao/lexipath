use eframe::egui;

use crate::audio::SystemSpeaker;
use crate::display_text::safe_ipa;
use crate::phonetics_catalog;
use crate::phonetics_engine::{PhoneticPhase, PhoneticSession};
use crate::progress_store::ProgressStore;

pub struct IpaApp {
    lessons: Vec<crate::phonetics::PhoneticLesson>,
    day_index: usize,
    session: PhoneticSession,
    store: ProgressStore,
    speaker: SystemSpeaker,
    status: String,
    locked_today: bool,
}

impl IpaApp {
    pub fn load() -> anyhow::Result<Option<Self>> {
        let lessons = phonetics_catalog::lessons();
        let store = ProgressStore::open()?;
        let day_index = store.ipa_completed_days();
        if day_index >= lessons.len() {
            return Ok(None);
        }
        let locked_today = store.ipa_completed_today();
        let session = PhoneticSession::new(lessons[day_index].clone());
        Ok(Some(Self {
            lessons,
            day_index,
            session,
            store,
            speaker: SystemSpeaker,
            status: if locked_today {
                "今日音标课程已完成，可以手动进入下一天。".to_owned()
            } else {
                "每个示例必须先播放，测试最终 100% 才能完成今天课程。".to_owned()
            },
            locked_today,
        }))
    }

    pub fn current_label(&self) -> String {
        format!(
            "当前音标：第 {} / {} 天：{}",
            self.day_index + 1,
            self.lessons.len(),
            self.session.lesson().title
        )
    }

    pub fn locked_today(&self) -> bool {
        self.locked_today
    }

    pub fn continue_after_daily_limit(&mut self) {
        self.locked_today = false;
        self.session = PhoneticSession::new(self.lessons[self.day_index].clone());
        self.status = "已手动进入下一天音标课程。".to_owned();
    }

    pub fn update(&mut self, context: &egui::Context) -> bool {
        if self.locked_today && !self.store.ipa_completed_today() {
            self.locked_today = false;
            self.session = PhoneticSession::new(self.lessons[self.day_index].clone());
            self.status = "新的一天已经开始，可以继续音标课程。".to_owned();
        }

        let mut all_complete = false;

        egui::TopBottomPanel::top("ipa_header").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.strong("LexiPath IPA");
                ui.separator();
                ui.label(format!("第 {} / {} 天", self.day_index + 1, self.lessons.len()));
                ui.separator();
                ui.label(self.session.lesson().title);
            });
        });

        egui::TopBottomPanel::bottom("ipa_status").show(context, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(context, |ui| {
            ui.vertical_centered_justified(|ui| {
                if self.locked_today {
                    ui.heading("今日音标学习已完成");
                    ui.label("固定计划每天只开放一课音标；需要继续测试时，可以手动进入下一天。");
                    if ui.button("进入下一天音标").clicked() {
                        self.continue_after_daily_limit();
                    }
                    return;
                }

                match self.session.phase() {
                    PhoneticPhase::Exposure => self.show_exposure(ui),
                    PhoneticPhase::ListeningTest => self.show_test(ui),
                    PhoneticPhase::Complete => {
                        ui.heading("本日音标测试最终正确率 100%");
                        if ui.button("完成今天课程").clicked() {
                            if let Err(error) = self.store.complete_ipa_day(self.lessons.len()) {
                                self.status = format!("保存音标进度失败：{error}");
                                return;
                            }
                            self.day_index += 1;
                            if self.day_index >= self.lessons.len() {
                                all_complete = true;
                            } else {
                                self.session =
                                    PhoneticSession::new(self.lessons[self.day_index].clone());
                                self.locked_today = true;
                                self.status =
                                    "今日音标课程已完成，可以手动进入下一天。".to_owned();
                            }
                        }
                    }
                }
            });
        });

        all_complete
    }

    fn show_exposure(&mut self, ui: &mut egui::Ui) {
        let Some(item) = self.session.current_item().cloned() else {
            return;
        };
        ui.heading(egui::RichText::new(safe_ipa(item.symbol)).size(42.0));
        ui.label(item.hint);
        ui.add_space(12.0);
        ui.label(egui::RichText::new(item.example).size(28.0));
        ui.label(egui::RichText::new(safe_ipa(item.example_ipa)).size(21.0));
        if ui.button("▶ 播放英文示例").clicked() {
            match self.speaker.speak(item.example) {
                Ok(()) => {
                    self.session.mark_audio_played();
                    self.status = "正在播放英文示例。".to_owned();
                }
                Err(error) => self.status = error,
            }
        }
        let enabled = self.session.audio_played();
        if ui
            .add_enabled(enabled, egui::Button::new("继续"))
            .clicked()
        {
            self.session.advance_exposure();
        }
        if !enabled {
            ui.label("必须先播放当前英文示例。");
        }
    }

    fn show_test(&mut self, ui: &mut egui::Ui) {
        let Some(item) = self.session.current_item().cloned() else {
            return;
        };
        let Some((options, correct_index)) = self.session.test_options() else {
            return;
        };

        ui.heading("听音选择对应音标");
        if ui.button("▶ 播放英文示例").clicked() {
            match self.speaker.speak(item.example) {
                Ok(()) => {
                    self.session.mark_audio_played();
                    self.status = "听完后选择对应音标。".to_owned();
                }
                Err(error) => self.status = error,
            }
        }

        for (index, option) in options.into_iter().enumerate() {
            if ui
                .add_enabled(self.session.audio_played(), egui::Button::new(safe_ipa(&option)))
                .clicked()
            {
                self.status = if self.session.answer(index, correct_index) {
                    "正确。".to_owned()
                } else {
                    "错误，该音标会重新进入测试队列。".to_owned()
                };
            }
        }
        if !self.session.audio_played() {
            ui.label("请先播放英文示例。");
        }
    }
}
