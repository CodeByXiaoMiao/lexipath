use std::fs;

use eframe::egui;

use crate::audio::SystemSpeaker;
use crate::catalog::CourseCatalog;
use crate::course::{CoursePack, Lesson};
use crate::display_text::safe_ipa;
use crate::engine::{LearningSession, Phase};
use crate::practice::due_practice_session;
use crate::progress_store::ProgressStore;
use crate::validator::tokenize;

pub struct LexiPathApp {
    course: CoursePack,
    session: LearningSession,
    active_review_id: Option<u64>,
    progress: Option<ProgressStore>,
    speaker: SystemSpeaker,
    status: String,
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
            status: "按固定顺序完成学习。到期复习优先于新课。".to_owned(),
            course_finished: false,
        };
        app.load_next_available();
        app
    }

    pub fn lesson_count(&self) -> usize {
        self.course.lesson_count()
    }

    pub fn current_lesson_number(&self) -> usize {
        self.lesson_number_by_id(&self.session.lesson().id).unwrap_or(1)
    }

    pub fn current_lesson_label(&self) -> String {
        let number = self.current_lesson_number();
        format!("第 {number} / {} 课：{}", self.lesson_count(), self.session.lesson().title)
    }

    pub fn continue_after_daily_limit(&mut self) {
        if let Some(store) = &mut self.progress {
            if let Err(error) = store.enable_manual_new_units_today() {
                self.status = format!("保存手动进入下一天失败：{error}");
                return;
            }
        }
        self.load_next_available();
        self.status = "已手动进入下一天/后续新课；到期复习仍会优先。".to_owned();
    }

    pub fn jump_relative_lesson(&mut self, offset: isize) -> Result<String, String> {
        let total = self.lesson_count();
        if total == 0 {
            return Err("课程为空，无法切换进度。".to_owned());
        }
        let current = self.current_lesson_number();
        let target = if offset < 0 {
            current.saturating_sub(offset.unsigned_abs())
        } else {
            current.saturating_add(offset as usize)
        }
        .clamp(1, total);
        self.jump_to_lesson_number(target)
    }

    pub fn jump_to_lesson_number(&mut self, number: usize) -> Result<String, String> {
        let total = self.lesson_count();
        if total == 0 {
            return Err("课程为空，无法切换进度。".to_owned());
        }
        let target = number.clamp(1, total);
        let lesson = self
            .lesson_by_number(target)
            .ok_or_else(|| format!("找不到第 {target} 课。"))?
            .clone();

        if let Some(store) = &mut self.progress {
            store.data.current_lesson_id = Some(lesson.id.clone());
            store.data.course_complete = false;
            store
                .enable_manual_new_units_today()
                .map_err(|error| format!("保存进度切换失败：{error}"))?;
        }

        self.session = LearningSession::new(lesson.clone());
        self.active_review_id = None;
        self.course_finished = false;
        self.status = format!("已切换到第 {target} / {total} 课：{}。", lesson.title);
        Ok(self.status.clone())
    }

    fn lesson_by_number(&self, number: usize) -> Option<&Lesson> {
        if number == 0 {
            return None;
        }
        self.course
            .stages
            .iter()
            .flat_map(|stage| stage.lessons.iter())
            .nth(number - 1)
    }

    fn lesson_number_by_id(&self, lesson_id: &str) -> Option<usize> {
        self.course
            .stages
            .iter()
            .flat_map(|stage| stage.lessons.iter())
            .position(|lesson| lesson.id == lesson_id)
            .map(|index| index + 1)
    }

    fn load_next_available(&mut self) {
        if let Some(review) = self
            .progress
            .as_ref()
            .and_then(|store| store.next_due_review())
            .cloned()
        {
            if let Some(lesson) = self.course.lesson_by_id(&review.lesson_id) {
                self.session = due_practice_session(lesson.clone());
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
            self.status = if lesson.is_stage_assessment() {
                "Ogden 已学完。完成阶段总结长文和理解测试后才能进入 Oxford。".to_owned()
            } else {
                "开始当前固定课程。".to_owned()
            };
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
        ui.label(egui::RichText::new(safe_ipa(&word.ipa)).size(22.0));
        ui.label(egui::RichText::new(&word.meaning).size(20.0));
        ui.separator();
        if ui.button("▶ 播放单词").clicked() {
            self.speak(&word.text);
            self.session.mark_word_audio_played();
        }
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
        let is_assessment = lesson.is_stage_assessment();
        ui.heading(&lesson.reading.title);
        if is_assessment {
            let word_count = tokenize(&lesson.full_reading_text()).len();
            ui.label(format!(
                "Ogden 阶段结业阅读：约 {word_count} 词，{} 道理解题。全文只使用已经学过的词。",
                lesson.reading.questions.len()
            ));
        } else {
            ui.label("正文已通过累计已学词白名单和内容质量检查。");
        }
        ui.separator();

        let reserved_height = if is_assessment { 145.0 } else { 120.0 };
        let reading_height = (ui.available_height() - reserved_height).max(150.0);
        egui::ScrollArea::vertical()
            .max_height(reading_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, sentence) in lesson.reading.sentences.iter().enumerate() {
                    if is_assessment && index % 12 == 0 {
                        if index > 0 {
                            ui.add_space(10.0);
                        }
                        ui.strong(format!("第 {} 段", index / 12 + 1));
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(sentence).size(19.0));
                        if ui.small_button("▶").clicked() {
                            self.speak(sentence);
                        }
                    });
                }
            });

        ui.separator();
        if ui.button("▶ 播放全文").clicked() {
            self.speak(&lesson.full_reading_text());
            self.session.mark_reading_audio_played();
        }
        let enabled = self.session.reading_audio_played();
        let next_label = if is_assessment {
            "进入阶段结业测试"
        } else {
            "进入阅读理解"
        };
        if ui
            .add_enabled(enabled, egui::Button::new(next_label))
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
        ui.heading(if self.session.lesson().is_stage_assessment() {
            "阶段结业阅读理解"
        } else {
            "阅读理解"
        });
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
                        format!(
                            "错误：你选择了「{}」。正确答案是「{}」。本题会重新出现，不能带错通过。",
                            option, question.options[question.correct_index]
                        )
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
        let correct_text = options.get(correct_index).cloned().unwrap_or_default();
        for (index, option) in options.into_iter().enumerate() {
            if ui
                .add_enabled(enabled, egui::Button::new(&option))
                .clicked()
            {
                let result = self.session.answer_current(index, correct_index);
                self.status = if result.correct {
                    "正确。".to_owned()
                } else {
                    format!(
                        "错误：你选择了「{}」。正确答案是「{}」。该项目仍留在待掌握队列。",
                        option, correct_text
                    )
                };
            }
        }
        if !enabled {
            ui.label("请先播放当前英文。");
        }
    }

    fn show_complete(&mut self, ui: &mut egui::Ui) {
        let is_assessment = self.session.lesson().is_stage_assessment();
        ui.heading(if is_assessment {
            "Ogden 阶段总结阅读已通过"
        } else {
            "本项最终正确率 100%"
        });
        ui.label(format!(
            "首次作答正确率：{:.0}%",
            self.session.first_attempt_accuracy() * 100.0
        ));
        let button = if is_assessment {
            "完成并解锁 Oxford"
        } else {
            "保存并继续固定计划"
        };
        if ui.button(button).clicked() {
            self.commit_and_continue();
        }
    }

    fn show_finished(&mut self, ui: &mut egui::Ui) {
        ui.heading("当前计划已完成");
        ui.label("没有到期复习。未来复习到期后，重新打开软件即可继续。");
    }
}

impl eframe::App for LexiPathApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&self.course.title);
                ui.separator();
                ui.label(format!("阶段：{}", phase_name(self.session.phase())));
                if let Some(store) = &self.progress {
                    ui.separator();
                    ui.label(format!(
                        "已完成 {} / {}",
                        store.completed_count(),
                        self.course.lesson_count()
                    ));
                    ui.label(format!("到期复习 {}", store.due_count()));
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
