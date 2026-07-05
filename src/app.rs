use std::fs;

use eframe::egui;

use crate::audio::SystemSpeaker;
use crate::course::CoursePack;
use crate::engine::{LearningSession, Phase};
use crate::storage::Storage;

pub struct LexiPathApp {
    course_title: String,
    session: LearningSession,
    speaker: SystemSpeaker,
    storage: Option<Storage>,
    status: String,
    completion_saved: bool,
}

impl LexiPathApp {
    pub fn new(context: &eframe::CreationContext<'_>, course: CoursePack) -> Self {
        install_windows_font(&context.egui_ctx);
        let lesson = course
            .first_lesson()
            .expect("validated course must contain a lesson")
            .clone();
        let storage = Storage::open_portable().ok();

        Self {
            course_title: course.title,
            session: LearningSession::new(lesson),
            speaker: SystemSpeaker,
            storage,
            status: "请按固定顺序完成当前课程。".to_owned(),
            completion_saved: false,
        }
    }

    fn speak(&mut self, text: &str) {
        match self.speaker.speak(text) {
            Ok(()) => self.status = "正在播放英文。".to_owned(),
            Err(error) => self.status = error,
        }
    }

    fn show_learn_words(&mut self, ui: &mut egui::Ui) {
        let Some(word) = self.session.current_word().cloned() else {
            return;
        };

        ui.heading(&word.text);
        ui.label(egui::RichText::new(&word.ipa).size(22.0));
        ui.label(egui::RichText::new(&word.meaning).size(20.0));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("▶ 单词").clicked() {
                self.speak(&word.text);
                self.session.mark_word_audio_played();
            }
            ui.label("第一次学习必须播放单词发音。 ");
        });

        ui.horizontal(|ui| {
            ui.label(format!("词组：{}", word.phrase));
            if ui.small_button("▶").clicked() {
                self.speak(&word.phrase);
            }
        });
        ui.horizontal(|ui| {
            ui.label(format!("例句：{}", word.example));
            if ui.small_button("▶").clicked() {
                self.speak(&word.example);
            }
        });

        ui.add_space(20.0);
        let enabled = self.session.can_advance_word();
        if ui
            .add_enabled(enabled, egui::Button::new("继续"))
            .clicked()
        {
            self.session.advance_word();
            self.status.clear();
        }
        if !enabled {
            ui.label("播放单词发音后才能继续。");
        }
    }

    fn show_recognition(&mut self, ui: &mut egui::Ui) {
        let Some(index) = self.session.current_mastery_index() else {
            return;
        };
        let word = self.session.lesson().new_words[index].clone();
        let Some((options, correct_index)) = self.session.recognition_options() else {
            return;
        };

        ui.heading(&word.text);
        ui.label("选择正确的中文核心含义：");
        self.show_answer_buttons(ui, options, correct_index, true);
    }

    fn show_listening(&mut self, ui: &mut egui::Ui) {
        let Some(index) = self.session.current_mastery_index() else {
            return;
        };
        let text = self.session.lesson().new_words[index].text.clone();
        let Some((options, correct_index)) = self.session.listening_options() else {
            return;
        };

        ui.heading("听音识词");
        if ui.button("▶ 播放").clicked() {
            self.speak(&text);
            self.session.mark_current_audio_played();
        }
        ui.label("听完后选择你听到的单词：");
        self.show_answer_buttons(
            ui,
            options,
            correct_index,
            self.session.current_audio_played(),
        );
    }

    fn show_sentences(&mut self, ui: &mut egui::Ui) {
        let Some(sentence) = self.session.current_sentence().cloned() else {
            return;
        };
        let Some((options, correct_index)) = self.session.sentence_options() else {
            return;
        };

        ui.heading(&sentence.text);
        if ui.button("▶ 播放句子").clicked() {
            self.speak(&sentence.text);
            self.session.mark_current_audio_played();
        }
        ui.label("选择正确的句意：");
        self.show_answer_buttons(
            ui,
            options,
            correct_index,
            self.session.current_audio_played(),
        );
    }

    fn show_reading(&mut self, ui: &mut egui::Ui) {
        let lesson = self.session.lesson().clone();
        ui.heading(&lesson.reading.title);
        ui.label("本课阅读只包含已学词和本课新词。");
        ui.separator();

        for sentence in &lesson.reading.sentences {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(sentence).size(19.0));
                if ui.small_button("▶").clicked() {
                    self.speak(sentence);
                }
            });
        }

        ui.add_space(16.0);
        if ui.button("▶ 播放全文").clicked() {
            self.speak(&lesson.full_reading_text());
            self.session.mark_reading_audio_played();
        }

        let enabled = self.session.reading_audio_played();
        if ui
            .add_enabled(enabled, egui::Button::new("完成阅读并进入理解测试"))
            .clicked()
        {
            self.session.finish_reading();
        }
        if !enabled {
            ui.label("必须至少播放一次全文，才能进入理解测试。");
        }
    }

    fn show_comprehension(&mut self, ui: &mut egui::Ui) {
        let Some(question) = self.session.current_question().cloned() else {
            return;
        };
        ui.heading("阅读理解");
        ui.label(egui::RichText::new(&question.prompt).size(19.0));

        for (index, option) in question.options.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.small_button("▶").clicked() {
                    self.speak(option);
                }
                if ui.button(option).clicked() {
                    let result = self
                        .session
                        .answer_current(index, question.correct_index);
                    self.status = if result.correct {
                        "正确。".to_owned()
                    } else {
                        "错误，本题会保留并重新出现。".to_owned()
                    };
                }
            });
        }
    }

    fn show_answer_buttons(
        &mut self,
        ui: &mut egui::Ui,
        options: Vec<String>,
        correct_index: usize,
        enabled: bool,
    ) {
        for (index, option) in options.into_iter().enumerate() {
            if ui
                .add_enabled(enabled, egui::Button::new(option))
                .clicked()
            {
                let result = self.session.answer_current(index, correct_index);
                self.status = if result.correct {
                    "正确。".to_owned()
                } else {
                    "错误，该项目会回到待掌握队列。".to_owned()
                };
            }
        }
        if !enabled {
            ui.label("请先播放当前英文。");
        }
    }

    fn show_complete(&mut self, ui: &mut egui::Ui) {
        ui.heading("当前学习单元已完成");
        ui.label("所有必测项目最终均已达到 100%。");
        ui.label(format!(
            "首次作答正确率：{:.0}%（仅用于记录，不影响 100% 通过规则）",
            self.session.first_attempt_accuracy() * 100.0
        ));

        if !self.completion_saved {
            if let Some(storage) = &self.storage {
                let _ = storage.save_lesson_complete(
                    &self.session.lesson().id,
                    self.session.first_attempt_accuracy(),
                );
            }
            self.completion_saved = true;
        }
    }
}

impl eframe::App for LexiPathApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.strong(&self.course_title);
                ui.separator();
                ui.label(format!("当前：{}", phase_name(self.session.phase())));
            });
        });

        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(context, |ui| {
            ui.vertical_centered_justified(|ui| match self.session.phase() {
                Phase::LearnWords => self.show_learn_words(ui),
                Phase::Recognition => self.show_recognition(ui),
                Phase::Listening => self.show_listening(ui),
                Phase::Sentences => self.show_sentences(ui),
                Phase::Reading => self.show_reading(ui),
                Phase::Comprehension => self.show_comprehension(ui),
                Phase::Complete => self.show_complete(ui),
            });
        });
    }
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::LearnWords => "新词学习",
        Phase::Recognition => "词义识别",
        Phase::Listening => "听音识词",
        Phase::Sentences => "句子训练",
        Phase::Reading => "零生词阅读",
        Phase::Comprehension => "阅读理解",
        Phase::Complete => "完成",
    }
}

fn install_windows_font(context: &egui::Context) {
    let candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\simhei.ttf",
    ];

    for path in candidates {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "windows-cjk".to_owned(),
            egui::FontData::from_owned(bytes).into(),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, "windows-cjk".to_owned());
        }
        context.set_fonts(fonts);
        break;
    }
}
