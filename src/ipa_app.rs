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

const IPA_SURFACE: egui::Color32 = egui::Color32::from_rgb(30, 41, 59);
const IPA_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(15, 23, 42);
const IPA_TEXT: egui::Color32 = egui::Color32::from_rgb(241, 245, 249);
const IPA_MUTED: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);
const IPA_ACCENT: egui::Color32 = egui::Color32::from_rgb(45, 212, 191);

impl IpaApp {
    pub fn load() -> anyhow::Result<Option<Self>> {
        let lessons = phonetics_catalog::lessons();
        let store = ProgressStore::open()?;
        if store.ipa_completed_days() >= lessons.len()
            && store.data.ipa_active_day_number.is_none()
        {
            return Ok(None);
        }
        let day_index = store
            .ipa_current_day_number(lessons.len())
            .saturating_sub(1);
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

    pub fn load_at_day_number(day_number: usize) -> anyhow::Result<Self> {
        let lessons = phonetics_catalog::lessons();
        let mut store = ProgressStore::open()?;
        let target = day_number.clamp(1, lessons.len().max(1));
        store.set_ipa_current_day_number(target, lessons.len())?;
        let day_index = target.saturating_sub(1);
        let session = PhoneticSession::new(lessons[day_index].clone());
        Ok(Self {
            lessons,
            day_index,
            session,
            store,
            speaker: SystemSpeaker,
            status: format!("已切换到第 {target} 天音标课程。"),
            locked_today: false,
        })
    }

    pub fn total_day_count() -> usize {
        phonetics_catalog::lessons().len()
    }

    pub fn current_day_number(&self) -> usize {
        self.day_index + 1
    }

    pub fn current_label(&self) -> String {
        format!(
            "当前音标：第 {} / {} 天：{}",
            self.current_day_number(),
            self.lessons.len(),
            self.session.lesson().title
        )
    }

    #[allow(dead_code)]
    pub fn locked_today(&self) -> bool {
        self.locked_today
    }

    pub fn continue_after_daily_limit(&mut self) {
        self.locked_today = false;
        self.session = PhoneticSession::new(self.lessons[self.day_index].clone());
        self.status = "已手动进入下一天音标课程。".to_owned();
    }

    #[allow(dead_code)]
    pub fn jump_relative_day(&mut self, offset: isize) -> Result<String, String> {
        let total = self.lessons.len();
        if total == 0 {
            return Err("音标课程为空，无法切换。".to_owned());
        }
        let current = self.current_day_number();
        let target = if offset < 0 {
            current.saturating_sub(offset.unsigned_abs())
        } else {
            current.saturating_add(offset as usize)
        }
        .clamp(1, total);
        self.jump_to_day_number(target)
    }

    pub fn jump_to_day_number(&mut self, day_number: usize) -> Result<String, String> {
        let total = self.lessons.len();
        if total == 0 {
            return Err("音标课程为空，无法切换。".to_owned());
        }
        let target = day_number.clamp(1, total);
        self.store
            .set_ipa_current_day_number(target, total)
            .map_err(|error| format!("保存音标进度失败：{error}"))?;
        self.day_index = target - 1;
        self.session = PhoneticSession::new(self.lessons[self.day_index].clone());
        self.locked_today = false;
        self.status = format!(
            "已切换到第 {target} / {total} 天音标：{}。",
            self.session.lesson().title
        );
        Ok(self.status.clone())
    }

    pub fn update(&mut self, context: &egui::Context) -> bool {
        if self.locked_today && !self.store.ipa_completed_today() {
            self.locked_today = false;
            self.session = PhoneticSession::new(self.lessons[self.day_index].clone());
            self.status = "新的一天已经开始，可以继续音标课程。".to_owned();
        }

        let mut all_complete = false;

        egui::TopBottomPanel::bottom("ipa_status")
            .frame(
                egui::Frame::new()
                    .fill(IPA_SURFACE)
                    .inner_margin(egui::Margin::symmetric(22, 9)),
            )
            .show(context, |ui| {
                ui.label(egui::RichText::new(&self.status).color(IPA_MUTED));
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(IPA_BACKGROUND))
            .show(context, |ui| {
            let available = ui.available_size();
            let card_width = available.x.min(760.0);
            let top_space = ((available.y - 520.0).max(0.0) * 0.5).min(96.0);
            ui.add_space(top_space);
            ui.vertical_centered(|ui| {
                ui.set_width(card_width);
                egui::Frame::new()
                    .fill(IPA_SURFACE)
                    .corner_radius(egui::CornerRadius::same(18))
                    .inner_margin(egui::Margin::same(28))
                    .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "音标训练  ·  第 {} / {} 天  ·  {}",
                            self.current_day_number(),
                            self.lessons.len(),
                            self.session.lesson().title
                        ))
                        .size(13.0)
                        .color(IPA_MUTED),
                    );
                    ui.add_space(12.0);
                if self.locked_today {
                    ui.heading(egui::RichText::new("今日音标学习已完成").color(IPA_TEXT));
                    ui.label(egui::RichText::new("固定计划每天只开放一课音标；需要继续测试时，可以手动进入下一天。").color(IPA_MUTED));
                    if ui.button("进入下一天音标").clicked() {
                        self.continue_after_daily_limit();
                    }
                    return;
                }

                match self.session.phase() {
                    PhoneticPhase::Exposure => self.show_exposure(ui),
                    PhoneticPhase::ListeningTest => self.show_test(ui),
                    PhoneticPhase::Complete => {
                        ui.heading(egui::RichText::new("本日音标测试最终正确率 100%").color(IPA_TEXT));
                        if ui.button("完成今天课程").clicked() {
                            let completed_day = self.current_day_number();
                            let total_days = self.lessons.len();
                            if let Err(error) =
                                self.store.complete_ipa_day(completed_day, total_days)
                            {
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
        });

        all_complete
    }

    fn show_exposure(&mut self, ui: &mut egui::Ui) {
        let Some(item) = self.session.current_item().cloned() else {
            return;
        };
        ui.heading(egui::RichText::new(safe_ipa(item.symbol)).size(42.0).color(IPA_ACCENT));
        ui.label(egui::RichText::new(item.hint).color(IPA_MUTED));
        ui.add_space(12.0);
        ui.label(egui::RichText::new(item.example).size(28.0));
        ui.label(egui::RichText::new(safe_ipa(item.example_ipa)).size(21.0));
        if ui
            .add(
                egui::Button::new("▶  播放英文示例")
                    .fill(IPA_ACCENT)
                    .corner_radius(egui::CornerRadius::same(10))
                    .min_size(egui::vec2(150.0, 40.0)),
            )
            .clicked()
        {
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

        ui.heading(egui::RichText::new("听音选择对应音标").color(IPA_TEXT));
        if ui
            .add(
                egui::Button::new("▶  播放英文示例")
                    .fill(IPA_ACCENT)
                    .corner_radius(egui::CornerRadius::same(10))
                    .min_size(egui::vec2(150.0, 40.0)),
            )
            .clicked()
        {
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
                .add_enabled(
                    self.session.audio_played(),
                    egui::Button::new(safe_ipa(&option)),
                )
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
