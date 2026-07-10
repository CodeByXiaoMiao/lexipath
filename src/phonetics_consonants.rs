use crate::phonetics::{PhoneticItem, PhoneticLesson};

const D8: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/t/", example: "tea", example_ipa: "/ti:/", hint: "清辅音" },
    PhoneticItem { symbol: "/d/", example: "day", example_ipa: "/dei/", hint: "浊辅音" },
    PhoneticItem { symbol: "/k/", example: "cat", example_ipa: "/kat/", hint: "清辅音" },
    PhoneticItem { symbol: "/g/", example: "go", example_ipa: "/gou/", hint: "浊辅音" },
];
const D9: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/f/", example: "food", example_ipa: "/fu:d/", hint: "清辅音" },
    PhoneticItem { symbol: "/v/", example: "very", example_ipa: "/'veri/", hint: "浊辅音" },
    PhoneticItem { symbol: "/th/", example: "three", example_ipa: "/thri:/", hint: "清辅音" },
    PhoneticItem { symbol: "/th/", example: "this", example_ipa: "/this/", hint: "浊辅音" },
];
const D10: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/s/", example: "see", example_ipa: "/si:/", hint: "清辅音" },
    PhoneticItem { symbol: "/z/", example: "zoo", example_ipa: "/zu:/", hint: "浊辅音" },
    PhoneticItem { symbol: "/sh/", example: "she", example_ipa: "/shi:/", hint: "清辅音" },
    PhoneticItem { symbol: "/zh/", example: "vision", example_ipa: "/'vizhen/", hint: "浊辅音" },
];
const D11: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/h/", example: "he", example_ipa: "/hi:/", hint: "清辅音" },
    PhoneticItem { symbol: "/ch/", example: "chair", example_ipa: "/cher/", hint: "清辅音" },
    PhoneticItem { symbol: "/j/", example: "job", example_ipa: "/ja:b/", hint: "浊辅音" },
];
const D12: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/m/", example: "man", example_ipa: "/man/", hint: "鼻音" },
    PhoneticItem { symbol: "/n/", example: "no", example_ipa: "/nou/", hint: "鼻音" },
    PhoneticItem { symbol: "/ng/", example: "sing", example_ipa: "/sing/", hint: "鼻音" },
];
const D13: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/l/", example: "look", example_ipa: "/luk/", hint: "舌侧音" },
    PhoneticItem { symbol: "/r/", example: "red", example_ipa: "/red/", hint: "卷舌音" },
    PhoneticItem { symbol: "/y/", example: "yes", example_ipa: "/yes/", hint: "半元音" },
    PhoneticItem { symbol: "/w/", example: "we", example_ipa: "/wi:/", hint: "半元音" },
];
const D14: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/'/", example: "teacher", example_ipa: "/'ti:cher/", hint: "主重音在其后的音节" },
    PhoneticItem { symbol: "/,/", example: "afternoon", example_ipa: "/,after'nu:n/", hint: "次重音在其后的音节" },
    PhoneticItem { symbol: "/e/", example: "about", example_ipa: "/e'baut/", hint: "非重读音节常见弱读" },
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
