use std::fs;

use eframe::egui;

use crate::audio::SystemSpeaker;
use crate::catalog::CourseCatalog;
use crate::course::CoursePack;
use crate::engine::{LearningSession, Phase};
use crate::progress_store::ProgressStore;
use crate::shell::DesktopShell;

pub struct LexiPathApp {
    course: CoursePack,
    session: LearningSession,
    active_review_id: Option<u64>,
    progress: Option<ProgressStore>,
    speaker: SystemSpeaker,
    shell: DesktopShell,
    status: String,
    compact: bool,
    course_finished: bool,
}

impl LexiPathApp {
    pub fn new(context: &eframe::CreationContext<'_>, course: CoursePack) -> Self {
        install_windows_font(&context.egui_ctx);
        let first = course
            .first_lesson()
            .expect("validated course must contain a lesson")
            .clone();
        let mut progress = ProgressStore::open().ok();

        if let Some(store) = &mut progress {
            if store.current_lesson_id().is_none() && !store.data.course_complete {
                let _ = store.set_current_lesson_id(&first.id);
            }
        }

        let mut app = Self {
            course,
            session: LearningSession::new(first),
            active_review_id: None,
            progress,
            speaker: SystemSpeaker,
            shell: DesktopShell::new(),
            status: "按固定顺序完成学习。到期复习优先于新课。".to_owned(),
            compact: false,
            course_finished: false,
        };
        app.load_next_available();
        app
    }

    fn load_next_available(&mut self) {
        if let Some(review) = self
            .progress
            .as_ref()
            .and_then(|store| store.next_due_review())
            .cloned()
        {
            if let Some(lesson) = self.course.lesson_by_id(&review.lesson_id) {
                self.session = LearningSession::new(lesson.clone());
                self.active_review_id = Some(review.id);
                self.course_finished = false;
                self.status = format!("正在完成第 {} 次到期复习。", review.step + 1);
                return;
            }
        }

        let course_complete = self
            .progress
            .as_ref()
            .map(|store| store.data.course_complete)
            .unwrap_or(false);
        if course_complete {
            self.course_finished = true;
            self.active_review_id = None;
            self.status = "主课程已完成，当前没有到期复习。".to_owned();
            return;
        }

        let lesson_id = self
            .progress
            .as_ref()
            .and_then(|store| store.current_lesson_id())
            .map(str::to_owned)
            .or_else(|| self.course.first_lesson().map(|lesson| lesson.id.clone()));

        if let Some(lesson) = lesson_id
            .as_deref()
            .and_then(|id| self.course.lesson_by_id(id))
        {
            self.session = LearningSession::new(lesson.clone());
            self.active_review_id = None;
            self.course_finished = false;
            self.status = "开始当前固定课程。".to_owned();
        }
    }

    fn commit_and_continue(&mut self) {
        let lesson_id = self.session.lesson().id.clone();
        let accuracy = self.session.first_attempt_accuracy();

        if let Some(store) = &mut self.progress {
            if let Some(review_id) = self.active_review_id.take() {
                if let Err(error) = store.complete_review(review_id) {
                    self.status = format!("保存复习失败：{error}");
                    return;
                }
            } else {
                if let Err(error) = store.complete_lesson(&lesson_id, accuracy) {
                    self.status = format!("保存课程失败：{error}");
                    return;
                }

                if let Some(next) = self.course.next_lesson(&lesson_id) {
                    if let Err(error) = store.set_current_lesson_id(&next.id) {
                        self.status = format!("保存下一课失败：{error}");
                        return;
                    }
                } else {
                    store.data.current_lesson_id = None;
                    store.data.course_complete = true;
                    if let Err(error) = store.save() {
                        self.status = format!("保存课程完成状态失败：{error}");
                        return;
                    }
                }
            }
        }

        self.load_next_available();
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
        if ui.button("▶ 播放单词").clicked() {
            self.speak(&word.text);
            self.session.mark_word_audio_played();
        }
        if !self.compact {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("词组：{}", word.phrase));
                if ui.small_button("▶").clicked() {
                    self.speak(&word.phrase);
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("例句：{}", word.example));
                if ui.small_button("▶").clicked() {
                    self.speak(&word.example);
                }
            });
        }
        let enabled = self.session.can_advance_word();
        if ui
            .add_enabled(enabled, egui::Button::new("继续"))
            .clicked()
        {
            self.session.advance_word();
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
        let Some((options, correct)) = self.session.recognition_options() else {
            return;
        };
        ui.heading(&word.text);
        ui.label("选择正确的核心含义：");
        self.show_answer_buttons(ui, options, correct, true);
    }

    fn show_listening(&mut self, ui: &mut egui::Ui) {
        let Some(index) = self.session.current_mastery_index() else {
            return;
        };
        let text = self.session.lesson().new_words[index].text.clone();
        let Some((options, correct)) = self.session.listening_options() else {
            return;
        };
        ui.heading("听音识词");
        if ui.button("▶ 播放").clicked() {
            self.speak(&text);
            self.session.mark_current_audio_played();
        }
        self.show_answer_buttons(
            ui,
            options,
            correct,
            self.session.current_audio_played(),
        );
    }

    fn show_sentences(&mut self, ui: &mut egui::Ui) {
        let Some(sentence) = self.session.current_sentence().cloned() else {
            return;
        };
        let Some((options, correct)) = self.session.sentence_options() else {
            return;
        };
        ui.heading(&sentence.text);
        if ui.button("▶ 播放句子").clicked() {
            self.speak(&sentence.text);
            self.session.mark_current_audio_played();
        }
        self.show_answer_buttons(
            ui,
            options,
            correct,
            self.session.current_audio_played(),
        );
    }

    fn show_reading(&mut self, ui: &mut egui::Ui) {
        let lesson = self.session.lesson().clone();
        ui.heading(&lesson.reading.title);
        ui.label("正文已通过累计已学词白名单校验。");
        ui.separator();
        for sentence in &lesson.reading.sentences {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(sentence).size(19.0));
                if ui.small_button("▶").clicked() {
                    self.speak(sentence);
                }
            });
        }
        if ui.button("▶ 播放全文").clicked() {
            self.speak(&lesson.full_reading_text());
            self.session.mark_reading_audio_played();
        }
        let enabled = self.session.reading_audio_played();
        if ui
            .add_enabled(enabled, egui::Button::new("进入阅读理解"))
            .clicked()
        {
            self.session.finish_reading();
        }
        if !enabled {
            ui.label("必须至少播放一次全文。");
        }
    }

    fn show_comprehension(&mut self, ui: &mut egui::Ui) {
        let Some(question) = self.session.current_question().cloned() else {
            return;
        };
        ui.heading("阅读理解");
        ui.label(&question.prompt);
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
                        "错误，本题会重新出现，不能带错通过。".to_owned()
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
                    "错误，该项目仍留在待掌握队列。".to_owned()
                };
            }
        }
        if !enabled {
            ui.label("请先播放当前英文。");
        }
    }

    fn show_complete(&mut self, ui: &mut egui::Ui) {
        ui.heading("本项最终正确率 100%");
        ui.label(format!(
            "首次作答正确率：{:.0}%",
            self.session.first_attempt_accuracy() * 100.0
        ));
        if ui.button("保存并继续固定计划").clicked() {
            self.commit_and_continue();
        }
    }

    fn show_finished(&mut self, ui: &mut egui::Ui) {
        ui.heading("当前计划已完成");
        ui.label("没有到期复习。未来复习到期后，重新打开软件即可继续。");
    }

    fn apply_compact_mode(&mut self, context: &egui::Context) {
        self.compact = !self.compact;
        let size = if self.compact {
            egui::vec2(380.0, 240.0)
        } else {
            egui::vec2(620.0, 520.0)
        };
        context.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        context.send_viewport_cmd(egui::ViewportCommand::Focus);
    }
}

impl eframe::App for LexiPathApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if self.shell.compact_toggle_requested() {
            self.apply_compact_mode(context);
        }

        egui::TopBottomPanel::top("header").show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&self.course.title);
                ui.separator();
                ui.label(format!("阶段：{}", phase_name(self.session.phase())));
                if let Some(store) = &self.progress {
                    ui.separator();
                    ui.label(format!("已完成 {} / {}", store.completed_count(), self.course.lesson_count()));
                    ui.label(format!("到期复习 {}", store.due_count()));
                }
                if ui.small_button(if self.compact { "展开" } else { "紧凑" }).clicked() {
                    self.apply_compact_mode(context);
                }
            });
        });

        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(context, |ui| {
            ui.vertical_centered_justified(|ui| {
                if self.course_finished {
                    self.show_finished(ui);
                    return;
                }
                match self.session.phase() {
                    Phase::LearnWords => self.show_learn_words(ui),
                    Phase::Recognition => self.show_recognition(ui),
                    Phase::Listening => self.show_listening(ui),
                    Phase::Sentences => self.show_sentences(ui),
                    Phase::Reading => self.show_reading(ui),
                    Phase::Comprehension => self.show_comprehension(ui),
                    Phase::Complete => self.show_complete(ui),
                }
            });
        });
    }
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::LearnWords => "新词",
        Phase::Recognition => "词义",
        Phase::Listening => "听音",
        Phase::Sentences => "句子",
        Phase::Reading => "阅读",
        Phase::Comprehension => "理解",
        Phase::Complete => "完成",
    }
}

fn install_windows_font(context: &egui::Context) {
    for path in [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\simhei.ttf",
    ] {
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
