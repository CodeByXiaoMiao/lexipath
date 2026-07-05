use crate::phonetics::{PhoneticItem, PhoneticLesson};

const D1: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/iː/", example: "see", example_ipa: "/siː/", hint: "长元音" },
    PhoneticItem { symbol: "/ɪ/", example: "sit", example_ipa: "/sɪt/", hint: "短元音" },
    PhoneticItem { symbol: "/e/", example: "bed", example_ipa: "/bed/", hint: "短元音" },
];
const D2: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/æ/", example: "cat", example_ipa: "/kæt/", hint: "短元音" },
    PhoneticItem { symbol: "/ʌ/", example: "cup", example_ipa: "/kʌp/", hint: "短元音" },
    PhoneticItem { symbol: "/ɑː/", example: "car", example_ipa: "/kɑːr/", hint: "长元音" },
];
const D3: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/ɔː/", example: "law", example_ipa: "/lɔː/", hint: "长元音" },
    PhoneticItem { symbol: "/ʊ/", example: "book", example_ipa: "/bʊk/", hint: "短元音" },
    PhoneticItem { symbol: "/uː/", example: "food", example_ipa: "/fuːd/", hint: "长元音" },
];
const D4: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/ɜːr/", example: "bird", example_ipa: "/bɜːrd/", hint: "卷舌元音" },
    PhoneticItem { symbol: "/ər/", example: "teacher", example_ipa: "/ˈtiːtʃər/", hint: "弱读卷舌音" },
    PhoneticItem { symbol: "/ə/", example: "about", example_ipa: "/əˈbaʊt/", hint: "弱读央元音" },
];
const D5: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/eɪ/", example: "day", example_ipa: "/deɪ/", hint: "双元音" },
    PhoneticItem { symbol: "/aɪ/", example: "my", example_ipa: "/maɪ/", hint: "双元音" },
    PhoneticItem { symbol: "/ɔɪ/", example: "boy", example_ipa: "/bɔɪ/", hint: "双元音" },
];
const D6: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/aʊ/", example: "now", example_ipa: "/naʊ/", hint: "双元音" },
    PhoneticItem { symbol: "/oʊ/", example: "go", example_ipa: "/ɡoʊ/", hint: "双元音" },
    PhoneticItem { symbol: "/ɪr/", example: "near", example_ipa: "/nɪr/", hint: "卷舌组合" },
];
const D7: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/er/", example: "hair", example_ipa: "/her/", hint: "卷舌组合" },
    PhoneticItem { symbol: "/p/", example: "pen", example_ipa: "/pen/", hint: "清辅音" },
    PhoneticItem { symbol: "/b/", example: "book", example_ipa: "/bʊk/", hint: "浊辅音" },
];

pub const LESSONS: &[PhoneticLesson] = &[
    PhoneticLesson { id: "ipa-01", title: "第 1 天：前元音", items: D1 },
    PhoneticLesson { id: "ipa-02", title: "第 2 天：开口元音", items: D2 },
    PhoneticLesson { id: "ipa-03", title: "第 3 天：后元音", items: D3 },
    PhoneticLesson { id: "ipa-04", title: "第 4 天：弱读与卷舌元音", items: D4 },
    PhoneticLesson { id: "ipa-05", title: "第 5 天：双元音一", items: D5 },
    PhoneticLesson { id: "ipa-06", title: "第 6 天：双元音二", items: D6 },
    PhoneticLesson { id: "ipa-07", title: "第 7 天：元音复习与爆破音", items: D7 },
];
