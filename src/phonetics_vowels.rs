use crate::phonetics::{PhoneticItem, PhoneticLesson};

const D1: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/i:/", example: "see", example_ipa: "/si:/", hint: "长元音" },
    PhoneticItem { symbol: "/i/", example: "sit", example_ipa: "/sit/", hint: "短元音" },
    PhoneticItem { symbol: "/e/", example: "bed", example_ipa: "/bed/", hint: "短元音" },
];
const D2: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/a/", example: "cat", example_ipa: "/kat/", hint: "短元音" },
    PhoneticItem { symbol: "/u/", example: "cup", example_ipa: "/kup/", hint: "短元音" },
    PhoneticItem { symbol: "/a:/", example: "car", example_ipa: "/ka:r/", hint: "长元音" },
];
const D3: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/o:/", example: "law", example_ipa: "/lo:/", hint: "长元音" },
    PhoneticItem { symbol: "/u/", example: "book", example_ipa: "/buk/", hint: "短元音" },
    PhoneticItem { symbol: "/u:/", example: "food", example_ipa: "/fu:d/", hint: "长元音" },
];
const D4: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/er:r/", example: "bird", example_ipa: "/ber:rd/", hint: "卷舌元音" },
    PhoneticItem { symbol: "/er/", example: "teacher", example_ipa: "/'ti:cher/", hint: "弱读卷舌音" },
    PhoneticItem { symbol: "/e/", example: "about", example_ipa: "/e'baut/", hint: "弱读央元音" },
];
const D5: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/ei/", example: "day", example_ipa: "/dei/", hint: "双元音" },
    PhoneticItem { symbol: "/ai/", example: "my", example_ipa: "/mai/", hint: "双元音" },
    PhoneticItem { symbol: "/oi/", example: "boy", example_ipa: "/boi/", hint: "双元音" },
];
const D6: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/au/", example: "now", example_ipa: "/nau/", hint: "双元音" },
    PhoneticItem { symbol: "/ou/", example: "go", example_ipa: "/gou/", hint: "双元音" },
    PhoneticItem { symbol: "/ir/", example: "near", example_ipa: "/nir/", hint: "卷舌组合" },
];
const D7: &[PhoneticItem] = &[
    PhoneticItem { symbol: "/er/", example: "hair", example_ipa: "/her/", hint: "卷舌组合" },
    PhoneticItem { symbol: "/p/", example: "pen", example_ipa: "/pen/", hint: "清辅音" },
    PhoneticItem { symbol: "/b/", example: "book", example_ipa: "/buk/", hint: "浊辅音" },
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
