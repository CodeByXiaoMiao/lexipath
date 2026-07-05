#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_v2;
mod audio;
mod catalog;
mod catalog_daily;
mod catalog_import;
mod catalog_load;
mod catalog_polish;
mod catalog_quality;
mod catalog_repair;
mod catalog_semantic_templates;
mod catalog_template_overrides;
mod course;
mod course_finalize;
mod course_finalize_file;
mod daily_gate;
mod embedded_course;
mod engine;
mod fonts;
mod ipa_app;
mod phonetics;
mod phonetics_catalog;
mod phonetics_consonants;
mod phonetics_engine;
mod phonetics_vowels;
mod practice;
mod progress_data;
mod progress_ipa;
mod progress_lesson;
mod progress_query;
mod progress_review;
mod progress_store;
mod root_app;
mod scheduler;
mod shell;
mod validator;

use root_app::RootApp;

fn main() -> eframe::Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("--repair-dictionary") {
        if let Err(error) = catalog_repair::repair(&arguments[1..]) {
            eprintln!("dictionary repair failed: {error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("--import-catalog") {
        if let Err(error) = import_and_finalize_catalog(&arguments[1..]) {
            eprintln!("catalog import failed: {error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("--finalize-catalog") {
        if let Err(error) = course_finalize_file::run(&arguments[1..]) {
            eprintln!("catalog finalization failed: {error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }

    let mut course = catalog_load::load().expect("course catalog could not be loaded");
    if course.first_lesson().is_none() {
        panic!("course catalog contains no lesson");
    }
    course_finalize::finalize_course(&mut course)
        .expect("course catalog failed final content validation");

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([620.0, 520.0])
            .with_min_inner_size([360.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Reference",
        options,
        Box::new(move |context| {
            RootApp::new(context, course)
                .map(|app| Box::new(app) as Box<dyn eframe::App>)
                .map_err(|error| Box::<dyn std::error::Error + Send + Sync>::from(error))
        }),
    )
}

fn import_and_finalize_catalog(arguments: &[String]) -> anyhow::Result<()> {
    catalog_import::import_catalog(arguments)?;
    let output_index = arguments
        .iter()
        .position(|argument| argument == "--output")
        .ok_or_else(|| anyhow::anyhow!("--output is required"))?;
    let output = arguments
        .get(output_index + 1)
        .ok_or_else(|| anyhow::anyhow!("missing value after --output"))?
        .clone();
    course_finalize_file::run(&[
        "--input".to_owned(),
        output.clone(),
        "--output".to_owned(),
        output,
    ])
}
