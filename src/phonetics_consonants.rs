use crate::phonetics::{PhoneticItem, PhoneticLesson};

const D8: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/t/", example: "tea", example_ipa: "/tiː/", hint: "清辅音" },
    PhoneticItem { symbol: "/d/", example: "day", example_ipa: "/deɪ/", hint: "浊辅音" },
    PhoneticItem { symbol: "/k/", example: "cat", example_ipa: "/kæt/", hint: "清辅音" },
    PhoneticItem { symbol: "/ɡ/", example: "go", example_ipa: "/ɡoʊ/", hint: "浊辅音" },
];
const D9: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/f/", example: "food", example_ipa: "/fuːd/", hint: "清辅音" },
    PhoneticItem { symbol: "/v/", example: "very", example_ipa: "/ˈveri/", hint: "浊辅音" },
    PhoneticItem { symbol: "/θ/", example: "three", example_ipa: "/θriː/", hint: "清辅音" },
    PhoneticItem { symbol: "/ð/", example: "this", example_ipa: "/ðɪs/", hint: "浊辅音" },
];
const D10: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/s/", example: "see", example_ipa: "/siː/", hint: "清辅音" },
    PhoneticItem { symbol: "/z/", example: "zoo", example_ipa: "/zuː/", hint: "浊辅音" },
    PhoneticItem { symbol: "/ʃ/", example: "she", example_ipa: "/ʃiː/", hint: "清辅音" },
    PhoneticItem { symbol: "/ʒ/", example: "vision", example_ipa: "/ˈvɪʒən/", hint: "浊辅音" },
];
const D11: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/h/", example: "he", example_ipa: "/hiː/", hint: "清辅音" },
    PhoneticItem { symbol: "/tʃ/", example: "chair", example_ipa: "/tʃer/", hint: "清辅音" },
    PhoneticItem { symbol: "/dʒ/", example: "job", example_ipa: "/dʒɑːb/", hint: "浊辅音" },
];
const D12: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/m/", example: "man", example_ipa: "/mæn/", hint: "鼻音" },
    PhoneticItem { symbol: "/n/", example: "no", example_ipa: "/noʊ/", hint: "鼻音" },
    PhoneticItem { symbol: "/ŋ/", example: "sing", example_ipa: "/sɪŋ/", hint: "鼻音" },
];
const D13: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/l/", example: "look", example_ipa: "/lʊk/", hint: "舌侧音" },
    PhoneticItem { symbol: "/r/", example: "red", example_ipa: "/red/", hint: "卷舌音" },
    PhoneticItem { symbol: "/j/", example: "yes", example_ipa: "/jes/", hint: "半元音" },
    PhoneticItem { symbol: "/w/", example: "we", example_ipa: "/wiː/", hint: "半元音" },
];
const D14: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/ˈ/", example: "teacher", example_ipa: "/ˈtiːtʃər/", hint: "主重音在其后的音节" },
    PhoneticItem { symbol: "/ˌ/", example: "afternoon", example_ipa: "/ˌæftərˈnuːn/", hint: "次重音在其后的音节" },
    PhoneticItem { symbol: "/ə/", example: "about", example_ipa: "/əˈbaʊt/", hint: "非重读音节常见弱读" },
];

pub const LESSONS: &[PhoneticLesson] = &[
    PhoneticLesson { id: "ipa-08", title: "第 8 天：爆破音", items: D8 },
    PhoneticLesson { id: "ipa-09", title: "第 9 天：摩擦音一", items: D9 },
    PhoneticLesson { id: "ipa-10", title: "第 10 天：摩擦音二", items: D10 },
    PhoneticLesson { id: "ipa-11", title: "第 11 天：破擦音", items: D11 },
    PhoneticLesson { id: "ipa-12", title: "第 12 天：鼻音", items: D12 },
    PhoneticLesson { id: "ipa-13", title: "第 13 天：舌侧音与半元音", items: D13 },
    PhoneticLesson { id: "ipa-14", title: "第 14 天：音节与重音", items: D14 },
];
