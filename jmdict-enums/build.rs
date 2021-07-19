/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use json::JsonValue;

struct EnumVariant {
    code: &'static str,
    name: &'static str,
    enabled: bool,
}

fn v(code: &'static str, name: &'static str) -> EnumVariant {
    EnumVariant {
        code,
        name,
        enabled: true,
    }
}

impl EnumVariant {
    fn when(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }
}

struct Enum<'a> {
    name: &'static str,
    all_name: Option<&'static str>,
    doc: String,
    entities: Option<&'a JsonValue>,
    variants: Vec<EnumVariant>,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/entities.json");

    let entities_str = std::fs::read_to_string("data/entities.json").unwrap();
    let entities = json::parse(&entities_str).unwrap();

    let mut content = String::new();

    content.push_str(&process(Enum {
        name: "Dialect",
        all_name: None,
        doc: "Dialect of Japanese in which a certain vocabulary occurs.".into(),
        entities: Some(&entities["dial"]),
        variants: vec![
            v("bra", "Brazilian"),
            v("hob", "Hokkaido"),
            v("ksb", "Kansai"),
            v("ktb", "Kantou"),
            v("kyb", "Kyoto"),
            v("kyu", "Kyuushuu"),
            v("nab", "Nagano"),
            v("osb", "Osaka"),
            v("rkb", "Ryuukyuu"),
            v("thb", "Touhoku"),
            v("tsb", "Tosa"),
            v("tsug", "Tsugaru"),
        ],
    }));

    content.push_str(&process(Enum {
        name: "GlossLanguage",
        all_name: Some("AllGlossLanguage"),
        doc: "The language of a particular Gloss.".into(),
        entities: None,
        variants: vec![
            v("eng", "English").when(cfg!(feature = "translations-eng")),
            v("dut", "Dutch").when(cfg!(feature = "translations-dut")),
            v("fre", "French").when(cfg!(feature = "translations-fre")),
            v("ger", "German").when(cfg!(feature = "translations-ger")),
            v("hun", "Hungarian").when(cfg!(feature = "translations-hun")),
            v("rus", "Russian").when(cfg!(feature = "translations-rus")),
            v("slv", "Slovenian").when(cfg!(feature = "translations-slv")),
            v("spa", "Spanish").when(cfg!(feature = "translations-spa")),
            v("swe", "Swedish").when(cfg!(feature = "translations-swe")),
        ],
    }));

    content.push_str(&process(Enum {
        name: "GlossType",
        all_name: None,
        doc: "Type of gloss.".into(),
        entities: None,
        variants: vec![
            v("", "RegularTranslation"),
            v("expl", "Explanation"),
            v("fig", "FigurativeSpeech"),
            v("lit", "LiteralTranslation"),
            v("tm", "Trademark"),
        ],
    }));

    content.push_str(&process(Enum {
        name: "KanjiInfo",
        all_name: None,
        doc: "Information regarding a certain KanjiElement.".into(),
        entities: Some(&entities["ke_inf"]),
        variants: vec![
            v("ateji", "Ateji"),
            v("iK", "IrregularKanjiUsage"),
            v("ik", "IrregularKanaUsage"),
            v("io", "IrregularOkuriganaUsage"),
            v("oK", "OutdatedKanji"),
            v("rK", "RareKanjiForm"),
        ],
    }));

    content.push_str(&process(Enum {
        name: "PartOfSpeech",
        all_name: Some("AllPartOfSpeech"),
        doc: "Where a word can appear in a sentence for a particular Sense of the word.".into(),
        entities: Some(&entities["pos"]),
        variants: vec![
            v("adj-f", "NounOrVerbActingPrenominally"),
            v("adj-i", "Adjective"),
            v("adj-ix", "YoiAdjective"),
            v("adj-kari", "KariAdjective").when(cfg!(feature = "scope-archaic")),
            v("adj-ku", "KuAdjective").when(cfg!(feature = "scope-archaic")),
            v("adj-na", "AdjectivalNoun"),
            v("adj-nari", "NariAdjective").when(cfg!(feature = "scope-archaic")),
            v("adj-no", "NoAdjective"),
            v("adj-pn", "PreNounAdjectival"),
            v("adj-shiku", "ShikuAdjective").when(cfg!(feature = "scope-archaic")),
            v("adj-t", "TaruAdjective"),
            v("adv", "Adverb"),
            v("adv-to", "AdverbTakingToParticle"),
            v("aux", "Auxiliary"),
            v("aux-adj", "AuxiliaryAdjective"),
            v("aux-v", "AuxiliaryVerb"),
            v("conj", "Conjunction"),
            v("cop", "Copula"),
            v("ctr", "Counter"),
            v("exp", "Expression"),
            v("int", "Interjection"),
            v("n", "CommonNoun"),
            v("n-adv", "AdverbialNoun"),
            v("n-pr", "ProperNoun"),
            v("n-pref", "NounPrefix"),
            v("n-suf", "NounSuffix"),
            v("n-t", "TemporalNoun"),
            v("num", "Numeric"),
            v("pn", "Pronoun"),
            v("pref", "Prefix"),
            v("prt", "Particle"),
            v("suf", "Suffix"),
            v("unc", "Unclassified"),
            v("v-unspec", "UnspecifiedVerb"),
            v("v1", "IchidanVerb"),
            v("v1-s", "IchidanKureruVerb"),
            v("v2a-s", "NidanUVerb").when(cfg!(feature = "scope-archaic")),
            v("v2b-k", "UpperNidanBuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2b-s", "LowerNidanBuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2d-k", "UpperNidanDzuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2d-s", "LowerNidanDzuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2g-k", "UpperNidanGuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2g-s", "LowerNidanGuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2h-k", "UpperNidanFuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2h-s", "LowerNidanFuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2k-k", "UpperNidanKuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2k-s", "LowerNidanKuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2m-k", "UpperNidanMuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2m-s", "LowerNidanMuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2n-s", "LowerNidanNuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2r-k", "UpperNidanRuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2r-s", "LowerNidanRuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2s-s", "LowerNidanSuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2t-k", "UpperNidanTsuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2t-s", "LowerNidanTsuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2w-s", "LowerNidanUWeVerb").when(cfg!(feature = "scope-archaic")),
            v("v2y-k", "UpperNidanYuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2y-s", "LowerNidanYuVerb").when(cfg!(feature = "scope-archaic")),
            v("v2z-s", "LowerNidanZuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4b", "YodanBuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4g", "YodanGuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4h", "YodanFuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4k", "YodanKuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4m", "YodanMuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4n", "YodanNuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4r", "YodanRuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4s", "YodanSuVerb").when(cfg!(feature = "scope-archaic")),
            v("v4t", "YodanTsuVerb").when(cfg!(feature = "scope-archaic")),
            v("v5aru", "GodanAruVerb"),
            v("v5b", "GodanBuVerb"),
            v("v5g", "GodanGuVerb"),
            v("v5k", "GodanKuVerb"),
            v("v5k-s", "GodanIkuVerb"),
            v("v5m", "GodanMuVerb"),
            v("v5n", "GodanNuVerb"),
            v("v5r", "GodanRuVerb"),
            v("v5r-i", "IrregularGodanRuVerb"),
            v("v5s", "GodanSuVerb"),
            v("v5t", "GodanTsuVerb"),
            v("v5u", "GodanUVerb"),
            v("v5u-s", "IrregularGodanUVerb"),
            v("vi", "IntransitiveVerb"),
            v("vk", "KuruVerb"),
            v("vn", "IrregularGodanNuVerb"),
            v("vr", "IrregularGodanRuVerbWithPlainRiForm"),
            v("vs", "SuruVerb"),
            v("vs-c", "SuruPrecursorVerb"),
            v("vs-i", "IncludedSuruVerb"),
            v("vs-s", "SpecialSuruVerb"),
            v("vt", "TransitiveVerb"),
            v("vz", "IchidanZuruVerb"),
        ],
    }));

    content.push_str(&process(Enum {
        name: "ReadingInfo",
        all_name: None,
        doc: "Information regarding a certain ReadingElement.".into(),
        entities: Some(&entities["re_inf"]),
        variants: vec![
            v("gikun", "GikunOrJukujikun"),
            v("ik", "IrregularKanaUsage"),
            v("ok", "OutdatedKanaUsage"),
            v("uK", "UsuallyWrittenUsingKanjiAlone"),
        ],
    }));

    content.push_str(&process(Enum {
        name: "SenseInfo",
        all_name: None,
        doc: "Information regarding a certain Sense.".into(),
        entities: Some(&entities["misc"]),
        variants: vec![
            v("X", "XRated"),
            v("abbr", "Abbreviation"),
            v("arch", "Archaism"),
            v("char", "Character"),
            v("chn", "ChildrensLanguage"),
            v("col", "Colloquialism"),
            v("company", "CompanyName"),
            v("creat", "Creature"),
            v("dated", "DatedTerm"),
            v("dei", "Deity"),
            v("derog", "Derogatory"),
            v("doc", "Document"),
            v("ev", "Event"),
            v("fam", "FamiliarLanguage"),
            v("fem", "FemaleTermOrLanguage"),
            v("fict", "Fiction"),
            v("form", "FormalOrLiteraryTerm"),
            v("given", "GivenName"),
            v("group", "Group"),
            v("hist", "HistoricalTerm"),
            v("hon", "HonorificLanguage"),
            v("hum", "HumbleLanguage"),
            v("id", "IdiomaticExpression"),
            v("joc", "JocularTerm"),
            v("leg", "Legend"),
            v("m-sl", "MangaSlang"),
            v("male", "MaleTermOrLanguage"),
            v("myth", "Mythology"),
            v("net-sl", "InternetSlang"),
            v("obj", "Object"),
            v("obs", "ObsoleteTerm"),
            v("obsc", "ObscureTerm"),
            v("on-mim", "Onomatopoeia"),
            v("organization", "OrganizationName"),
            v("oth", "Other"),
            v("person", "PersonName"),
            v("place", "PlaceName"),
            v("poet", "PoeticalTerm"),
            v("pol", "PoliteLanguage"),
            v("product", "ProductName"),
            v("proverb", "Proverb"),
            v("quote", "Quotation"),
            v("rare", "Rare"),
            v("relig", "Religion"),
            v("sens", "Sensitive"),
            v("serv", "Service"),
            v("sl", "Slang"),
            v("station", "RailwayStation"),
            v("surname", "Surname"),
            v("uk", "UsuallyWrittenUsingKanaAlone"),
            v("unclass", "UnclassifiedName"),
            v("vulg", "VulgarTerm"),
            v("work", "WorkOfArt"),
            v("yoji", "Yojijukugo"),
        ],
    }));

    content.push_str(&process(Enum {
        name: "SenseTopic",
        all_name: None,
        doc: "Field of study where a certain Sense originates.".into(),
        entities: Some(&entities["field"]),
        variants: vec![
            v("Buddh", "Buddhism"),
            v("Christn", "Christianity"),
            v("MA", "MartialArts"),
            v("Shinto", "Shinto"),
            v("agric", "Agriculture"),
            v("anat", "Anatomy"),
            v("archeol", "Archeology"),
            v("archit", "Architecture"),
            v("art", "Art"),
            v("astron", "Astronomy"),
            v("audvid", "AudioVisual"),
            v("aviat", "Aviation"),
            v("baseb", "Baseball"),
            v("biochem", "Biochemistry"),
            v("biol", "Biology"),
            v("bot", "Botany"),
            v("bus", "Business"),
            v("chem", "Chemistry"),
            v("cloth", "Clothing"),
            v("comp", "Computing"),
            v("cryst", "Crystallography"),
            v("ecol", "Ecology"),
            v("econ", "Economics"),
            v("elec", "ElectricalEngineering"),
            v("electr", "Electronics"),
            v("embryo", "Embryology"),
            v("engr", "Engineering"),
            v("ent", "Entomology"),
            v("finc", "Finance"),
            v("fish", "Fishing"),
            v("food", "Food"),
            v("gardn", "Gardening"),
            v("genet", "Genetics"),
            v("geogr", "Geography"),
            v("geol", "Geology"),
            v("geom", "Geometry"),
            v("go", "Go"),
            v("golf", "Golf"),
            v("gramm", "Grammar"),
            v("grmyth", "GreekMythology"),
            v("hanaf", "Hanafuda"),
            v("horse", "Horseracing"),
            v("law", "Law"),
            v("ling", "Linguistics"),
            v("logic", "Logic"),
            v("mahj", "Mahjong"),
            v("math", "Mathematics"),
            v("mech", "MechanicalEngineering"),
            v("med", "Medicine"),
            v("met", "Meteorology"),
            v("mil", "Military"),
            v("music", "Music"),
            v("ornith", "Ornithology"),
            v("paleo", "Paleontology"),
            v("pathol", "Pathology"),
            v("pharm", "Pharmacy"),
            v("phil", "Philosophy"),
            v("photo", "Photography"),
            v("physics", "Physics"),
            v("physiol", "Physiology"),
            v("print", "Printing"),
            v("psy", "Psychiatry"),
            v("psych", "Psychology"),
            v("rail", "Railway"),
            v("shogi", "Shogi"),
            v("sports", "Sports"),
            v("stat", "Statistics"),
            v("sumo", "Sumo"),
            v("telec", "Telecommunications"),
            v("tradem", "Trademark"),
            v("vidg", "VideoGame"),
            v("zool", "Zoology"),
        ],
    }));

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("generated.rs");
    std::fs::write(&dest_path, content).unwrap();
}

fn process(e: Enum) -> String {
    let mut lines = vec![];

    //render the corresponding fully-populated enum, if requested
    if let Some(all_name) = e.all_name {
        lines.push(process(Enum {
            name: all_name,
            all_name: None,
            doc: format!("{} This enum contains all possible variants, including those that have been disabled by compile-time flags in `enum {}`.", e.doc, e.name),
            entities: e.entities,
            variants: e.variants.iter().map(|v| EnumVariant{enabled: true, ..*v}).collect(),
        }));
    }

    //enum declaration
    lines.push(format!("/// {}", e.doc));
    lines.push("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]".into());
    lines.push(format!("pub enum {} {{", e.name));
    for v in e.variants.iter().filter(|v| v.enabled) {
        if let Some(ref entities) = e.entities {
            lines.push(format!("  ///{}", entities[v.code].as_str().unwrap()));
        }
        lines.push(format!("  {},", v.name));
    }
    lines.push("}\n".into());

    //start impl Enum
    lines.push(format!("impl Enum for {} {{", e.name));

    //fn code(&self) -> &str
    lines.push("    fn code(&self) -> &'static str {".into());
    lines.push("        match *self {".into());
    for v in e.variants.iter().filter(|v| v.enabled) {
        lines.push(format!(
            "            {}::{} => \"{}\",",
            e.name, v.name, v.code
        ));
    }
    lines.push("        }".into());
    lines.push("    }\n".into());

    //fn constant_name(&self) -> &str
    lines.push("    fn constant_name(&self) -> &'static str {".into());
    lines.push("        match *self {".into());
    for v in e.variants.iter().filter(|v| v.enabled) {
        lines.push(format!(
            "            {}::{} => \"{}\",",
            e.name, v.name, v.name
        ));
    }
    lines.push("        }".into());
    lines.push("    }\n".into());

    //fn from_code(&str) -> Self
    lines.push("    fn from_code(text: &str) -> Option<Self> {".into());
    lines.push("        match text {".into());
    for v in e.variants.iter().filter(|v| v.enabled) {
        lines.push(format!(
            "            \"{}\" => Some({}::{}),",
            v.code, e.name, v.name
        ));
    }
    lines.push("            _ => None,".into());
    lines.push("        }".into());
    lines.push("    }\n".into());

    //fn from_constant_name(&str) -> Self
    lines.push("    fn from_constant_name(text: &str) -> Option<Self> {".into());
    lines.push("        match text {".into());
    for v in e.variants.iter().filter(|v| v.enabled) {
        lines.push(format!(
            "            \"{}\" => Some({}::{}),",
            v.name, e.name, v.name
        ));
    }
    lines.push("            _ => None,".into());
    lines.push("        }".into());
    lines.push("    }\n".into());

    //fn all_variants() -> &'static [Self]
    lines.push("    fn all_variants() -> &'static [Self] {".into());
    lines.push("        &[".into());
    for v in e.variants.iter().filter(|v| v.enabled) {
        lines.push(format!("            {}::{},", e.name, v.name));
    }
    lines.push("        ]".into());
    lines.push("    }\n".into());

    //end impl Enum
    lines.push("}\n".into());

    //impl Display
    lines.push(format!("impl std::fmt::Display for {} {{", e.name));
    lines.push("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {".into());
    lines.push("        write!(f, \"{}\", self.constant_name())".into());
    lines.push("    }".into());
    lines.push("}\n".into());

    //impl EnumPayload
    lines.push(format!("impl EnumPayload for {} {{", e.name));
    lines.push("    fn to_u32(&self) -> u32 {".into());
    lines.push("        match *self {".into());
    for (idx, v) in e.variants.iter().filter(|v| v.enabled).enumerate() {
        lines.push(format!("            {}::{} => {},", e.name, v.name, idx));
    }
    lines.push("        }".into());
    lines.push("    }\n".into());
    lines.push("    fn from_u32(code: u32) -> Self {".into());
    lines.push("        match code {".into());
    for (idx, v) in e.variants.iter().filter(|v| v.enabled).enumerate() {
        lines.push(format!("            {} => {}::{},", idx, e.name, v.name));
    }
    lines.push(format!(
        "            _ => panic!(\"unknown {} code: {{}}\", code),",
        e.name
    ));
    lines.push("        }".into());
    lines.push("    }".into());
    lines.push("}\n".into());

    if let Some(all_name) = e.all_name {
        //impl TryFrom
        lines.push(format!(
            "impl std::convert::TryFrom<{}> for {} {{",
            all_name, e.name
        ));
        lines.push("    type Error = DisabledVariant;".into());
        lines.push(format!(
            "    fn try_from(value: {}) -> Result<{}, DisabledVariant> {{",
            all_name, e.name,
        ));
        lines.push("        match value {".into());
        for v in e.variants.iter() {
            if v.enabled {
                lines.push(format!(
                    "            {}::{} => Ok({}::{}),",
                    all_name, v.name, e.name, v.name
                ));
            } else {
                lines.push(format!(
                    "            {}::{} => Err(DisabledVariant),",
                    all_name, v.name
                ));
            }
        }
        lines.push("        }".into());
        lines.push("    }".into());
        lines.push("}\n".into());

        //impl From
        lines.push(format!(
            "impl std::convert::From<{}> for {} {{",
            e.name, all_name
        ));
        lines.push(format!("    fn from(value: {}) -> {} {{", e.name, all_name));
        lines.push("        match value {".into());
        for v in e.variants.iter().filter(|v| v.enabled) {
            lines.push(format!(
                "            {}::{} => {}::{},",
                e.name, v.name, all_name, v.name
            ));
        }
        lines.push("        }".into());
        lines.push("    }".into());
        lines.push("}\n".into());
    }

    lines.join("\n")
}
