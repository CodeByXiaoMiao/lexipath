use std::fs;

use eframe::egui;

pub fn install(context: &egui::Context) {
    let cjk = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\simhei.ttf",
    ]
    .into_iter()
    .find_map(|path| fs::read(path).ok());
    let ipa = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
    ]
    .into_iter()
    .find_map(|path| fs::read(path).ok());

    if cjk.is_none() && ipa.is_none() {
        return;
    }

    let mut fonts = egui::FontDefinitions::default();
    if let Some(bytes) = cjk {
        fonts.font_data.insert(
            "windows-cjk".to_owned(),
            egui::FontData::from_owned(bytes).into(),
        );
    }
    if let Some(bytes) = ipa {
        fonts.font_data.insert(
            "windows-ipa".to_owned(),
            egui::FontData::from_owned(bytes).into(),
        );
    }
    let has_cjk = fonts.font_data.contains_key("windows-cjk");
    let has_ipa = fonts.font_data.contains_key("windows-ipa");
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let family_fonts = fonts.families.entry(family).or_default();
        if has_cjk {
            family_fonts.insert(0, "windows-cjk".to_owned());
        }
        if has_ipa {
            family_fonts.insert(1.min(family_fonts.len()), "windows-ipa".to_owned());
        }
    }
    context.set_fonts(fonts);
}
