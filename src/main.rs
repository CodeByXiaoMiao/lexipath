#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod course;
mod engine;
mod storage;
mod validator;

use app::LexiPathApp;
use course::CoursePack;
use validator::validate_course;

fn main() -> eframe::Result<()> {
    let course = CoursePack::embedded().expect("embedded course could not be loaded");
    if course.first_lesson().is_none() {
        panic!("embedded course contains no lesson");
    }
    if let Err(errors) = validate_course(&course) {
        let details = errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        panic!("course validation failed:\n{details}");
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([620.0, 520.0])
            .with_min_inner_size([460.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Reference",
        options,
        Box::new(move |context| Ok(Box::new(LexiPathApp::new(context, course)))),
    )
}
