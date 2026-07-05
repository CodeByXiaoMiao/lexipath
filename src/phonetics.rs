#[derive(Debug, Clone)]
pub struct PhoneticItem {
    pub symbol: &'static str,
    pub example: &'static str,
    pub example_ipa: &'static str,
    pub hint: &'static str,
}

#[derive(Debug, Clone)]
pub struct PhoneticLesson {
    pub id: &'static str,
    pub title: &'static str,
    pub items: &'static [PhoneticItem],
}
