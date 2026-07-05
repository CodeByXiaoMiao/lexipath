#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_v2;
mod audio;
mod catalog;
mod catalog_daily;
mod catalog_import;
mod catalog_load;
mod course;
mod embedded_course;
mod engine;
mod practice;
mod progress_data;
mod progress_lesson;
mod progress_query;
mod progress_review;
mod progress_store;
mod scheduler;
mod shell;
mod validator;

use app_v2::LexiPathApp;
use validator::validate_course;

fn main() -> eframe::Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("--import-catalog") {
        if let Err(error) = catalog_import::import_catalog(&arguments[1..]) {
            eprintln!("catalog import failed: {error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }

    let course = catalog_load::load().expect("course catalog could not be loaded");
    if course.first_lesson().is_none() {
        panic!("course catalog contains no lesson");
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
            .with_min_inner_size([360.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Reference",
        options,
        Box::new(move |context| Ok(Box::new(LexiPathApp::new(context, course)))),
    )
}
