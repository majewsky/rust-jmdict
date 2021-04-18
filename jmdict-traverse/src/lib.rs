/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

//! Parsing utilities for the build and test phases of the `jmdict` crate.
//!
//! This code is in a separate crate because, if we put it in the `jmdict` crate itself, its
//! `build.rs` could not import it.
//!
//! # Compatibility promise
//!
//! **There is none.** Although this crate is published on crates.io for technical reasons, this
//! crate is internal to the `jmdict` crate. Its API may change at any time, including in
//! bugfix releases. Use the [API provided by the `jmdict` crate](https://docs.rs/jmdict/) instead.

use jmdict_enums::{
    AllGlossLanguage, AllPartOfSpeech, Dialect, Enum, GlossLanguage, GlossType, KanjiInfo,
    PartOfSpeech, Priority, PriorityInCorpus, ReadingInfo, SenseInfo, SenseTopic,
};
use json::JsonValue;
use std::convert::TryInto;

mod entrypack;
use entrypack::EntryPack;

pub struct RawEntry<'a> {
    pub ent_seq: u32,
    pub k_ele: Vec<RawKanjiElement<'a>>,
    pub r_ele: Vec<RawReadingElement<'a>>,
    pub sense: Vec<RawSense<'a>>,
}

pub struct RawKanjiElement<'a> {
    pub keb: &'a str,
    pub ke_inf: Vec<KanjiInfo>,
    pub ke_pri: Priority,
}

pub struct RawReadingElement<'a> {
    pub reb: &'a str,
    pub re_nokanji: bool,
    pub re_restr: Vec<&'a str>,
    pub re_inf: Vec<ReadingInfo>,
    pub re_pri: Priority,
}

pub struct RawSense<'a> {
    pub stagk: Vec<&'a str>,
    pub stagr: Vec<&'a str>,
    pub pos: Vec<PartOfSpeech>,
    pub xref: Vec<&'a str>,
    pub ant: Vec<&'a str>,
    pub field: Vec<SenseTopic>,
    pub misc: Vec<SenseInfo>,
    pub s_inf: Vec<&'a str>,
    pub lsource: Vec<RawLSource<'a>>,
    pub dial: Vec<Dialect>,
    pub gloss: Vec<RawGloss<'a>>,
}

pub struct RawLSource<'a> {
    //NOTE: We do not use the GlossLanguage enum for the lang attribute, because doing so would add
    //a very long tail of rare loanword source languages to that enum. (Also, we could not restrict
    //variants of GlossLanguage to feature flags in the way we currently do.)
    pub text: &'a str,
    pub lang: &'a str,
    pub is_partial: bool,
    pub is_wasei: bool,
}

pub struct RawGloss<'a> {
    //NOTE: g_gend and pri are not mapped since they do not actually occur in any entries
    pub text: &'a str,
    pub lang: GlossLanguage,
    pub g_type: GlossType,
}

///Strategy for processing a JMdict file.
pub trait Visitor {
    fn process_entry(&mut self, entry: &RawEntry);

    ///This is called once for each file that was read from disk. The build script uses this to
    ///generate `cargo:rerun-if-changed` directives.
    fn notify_data_file_path(&mut self, _path: &str) {}
}

///Options for traversing a JMdict file. This controls which entries the [Visitor] visits, and
///which parts of the entries it sees.
pub struct Options {
    pub is_db_minimal: bool,
    pub with_uncommon: bool,
    pub with_archaic: bool,
}

///Entry point for this file. All other functions are called directly or indirectly from this fn.
pub fn process_dictionary<V: Visitor>(v: &mut V, opts: Options) {
    let entrypack = EntryPack::locate_or_download();
    v.notify_data_file_path(&entrypack.path.to_string_lossy());

    for entry_str in entrypack.contents().split('\n') {
        if !entry_str.is_empty() {
            let entry_obj = json::parse(entry_str).unwrap();
            if let Some(entry_raw) = RawEntry::from_obj(&entry_obj, &opts) {
                if opts.is_db_minimal && entry_raw.ent_seq >= 1010000 {
                    //for db-minimal, only process entries from data/entries-100.json
                    return;
                }
                v.process_entry(&entry_raw);
            }
        }
    }
}

trait Object<'a>: Sized {
    fn from_obj(obj: &'a JsonValue, opts: &'_ Options) -> Option<Self>;

    fn collect(array: &'a JsonValue, opts: &'_ Options) -> Vec<Self> {
        assert!(array.is_null() || array.is_array());
        array
            .members()
            .filter_map(|obj| Self::from_obj(obj, opts))
            .collect()
    }

    fn collect_or_none(array: &'a JsonValue, opts: &'_ Options) -> Option<Vec<Self>> {
        let vec = Self::collect(array, opts);
        if vec.is_empty() {
            None
        } else {
            Some(vec)
        }
    }
}

impl<'a> Object<'a> for RawEntry<'a> {
    fn from_obj(obj: &'a JsonValue, opts: &'_ Options) -> Option<Self> {
        Some(Self {
            ent_seq: obj["n"].as_u32().unwrap(),
            k_ele: RawKanjiElement::collect(&obj["K"], opts),
            r_ele: RawReadingElement::collect_or_none(&obj["R"], opts)?,
            sense: RawSense::collect_or_none(&obj["S"], opts)?,
        })
    }
}

impl<'a> Object<'a> for RawKanjiElement<'a> {
    fn from_obj(obj: &'a JsonValue, opts: &'_ Options) -> Option<Self> {
        if !opts.with_uncommon && obj["p"].is_empty() {
            return None;
        }
        Some(Self {
            keb: obj["t"].as_str().unwrap(),
            ke_inf: Object::collect(&obj["i"], opts),
            ke_pri: parse_prio(Object::collect(&obj["p"], opts)),
        })
    }
}

impl<'a> Object<'a> for RawReadingElement<'a> {
    fn from_obj(obj: &'a JsonValue, opts: &'_ Options) -> Option<Self> {
        if !opts.with_uncommon && obj["p"].is_empty() {
            return None;
        }
        Some(Self {
            reb: obj["t"].as_str().unwrap(),
            re_nokanji: obj["n"].as_bool().unwrap_or(false),
            re_restr: Object::collect(&obj["r"], opts),
            re_inf: Object::collect(&obj["i"], opts),
            re_pri: parse_prio(Object::collect(&obj["p"], opts)),
        })
    }
}

fn parse_prio(markers: Vec<&str>) -> Priority {
    use PriorityInCorpus::*;
    let mut result = Priority {
        news: Absent,
        ichimango: Absent,
        loanwords: Absent,
        additional: Absent,
        frequency_bucket: 0,
    };
    for marker in markers {
        match marker {
            "news1" => result.news = merge_cprio(result.news, Primary),
            "news2" => result.news = merge_cprio(result.news, Secondary),
            "ichi1" => result.ichimango = merge_cprio(result.ichimango, Primary),
            "ichi2" => result.ichimango = merge_cprio(result.ichimango, Secondary),
            "gai1" => result.loanwords = merge_cprio(result.loanwords, Primary),
            "gai2" => result.loanwords = merge_cprio(result.loanwords, Secondary),
            "spec1" => result.additional = merge_cprio(result.additional, Primary),
            "spec2" => result.additional = merge_cprio(result.additional, Secondary),
            _ => match parse_freq_bucket(marker) {
                Some(bucket) => {
                    if result.frequency_bucket == 0 || result.frequency_bucket > bucket {
                        result.frequency_bucket = bucket;
                    }
                }
                None => {
                    panic!("unknown priority marker: {}", marker);
                }
            },
        };
    }
    result
}

fn merge_cprio(old: PriorityInCorpus, new: PriorityInCorpus) -> PriorityInCorpus {
    use PriorityInCorpus::*;
    match (old, new) {
        (Absent, _) => new,
        (_, Primary) => Primary,
        (Primary, _) => Primary,
        (Secondary, _) => Secondary,
    }
}

///Parses a frequency bucket marker for the news corpus, e.g. "nf18" => Some(18).
fn parse_freq_bucket(marker: &str) -> Option<u16> {
    //NOTE: This would be easier with a regex library, but I'm definitely not pulling in an entire
    //regex crate for just this one thing.

    let mut c = marker.chars();
    if c.next()? != 'n' {
        return None;
    }
    if c.next()? != 'f' {
        return None;
    }
    let tens = c.next()?.to_digit(10)? as u16;
    let ones = c.next()?.to_digit(10)? as u16;
    if c.next().is_some() {
        return None;
    }
    let result = 10 * tens + ones;

    //only nf01..nf48 are allowed
    if result == 0 || result > 48 {
        None
    } else {
        Some(result)
    }
}

impl<'a> Object<'a> for RawSense<'a> {
    fn from_obj(obj: &'a JsonValue, opts: &'_ Options) -> Option<Self> {
        let misc = Object::collect(&obj["m"], opts);
        if !opts.with_archaic && misc.contains(&SenseInfo::Archaism) {
            return None;
        }

        Some(Self {
            stagk: Object::collect(&obj["stagk"], opts),
            stagr: Object::collect(&obj["stagr"], opts),
            pos: Object::collect(&obj["p"], opts),
            xref: Object::collect(&obj["xref"], opts),
            ant: Object::collect(&obj["ant"], opts),
            field: Object::collect(&obj["f"], opts),
            misc,
            s_inf: Object::collect(&obj["i"], opts),
            lsource: Object::collect(&obj["L"], opts),
            dial: Object::collect(&obj["dial"], opts),
            gloss: Object::collect_or_none(&obj["G"], opts)?,
        })
    }
}

impl<'a> Object<'a> for RawLSource<'a> {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        let is_partial = match obj["type"].as_str().unwrap_or("full") {
            "full" => false,
            "part" => true,
            val => panic!("unknown ls_type: {}", val),
        };
        let is_wasei = match obj["wasei"].as_str().unwrap_or("n") {
            "n" => false,
            "y" => true,
            val => panic!("unknown ls_wasei: {}", val),
        };
        Some(Self {
            text: obj["t"].as_str().unwrap(),
            lang: obj["l"].as_str().unwrap_or("eng"),
            is_partial,
            is_wasei,
        })
    }
}

impl<'a> Object<'a> for RawGloss<'a> {
    fn from_obj(obj: &'a JsonValue, opts: &'_ Options) -> Option<Self> {
        Some(Self {
            text: obj["t"].as_str().unwrap(),
            lang: GlossLanguage::from_obj(&obj["l"], opts)?,
            g_type: optional_enum(&obj["g_type"], "", "GlossType"),
        })
    }
}

impl<'a> Object<'a> for &'a str {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        Some(obj.as_str().unwrap())
    }
}

impl<'a> Object<'a> for Dialect {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        Some(required_enum(obj, "Dialect"))
    }
}

impl<'a> Object<'a> for GlossLanguage {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        let lang: AllGlossLanguage = optional_enum(obj, "eng", "AllGlossLanguage");
        lang.try_into().ok()
    }
}

impl<'a> Object<'a> for KanjiInfo {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        Some(required_enum(obj, "KanjiInfo"))
    }
}

impl<'a> Object<'a> for PartOfSpeech {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        let lang: AllPartOfSpeech = optional_enum(obj, "eng", "AllPartOfSpeech");
        lang.try_into().ok()
    }
}

impl<'a> Object<'a> for ReadingInfo {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        Some(required_enum(obj, "ReadingInfo"))
    }
}

impl<'a> Object<'a> for SenseInfo {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        Some(required_enum(obj, "SenseInfo"))
    }
}

impl<'a> Object<'a> for SenseTopic {
    fn from_obj(obj: &'a JsonValue, _opts: &'_ Options) -> Option<Self> {
        Some(required_enum(obj, "SenseTopic"))
    }
}

fn optional_enum<E: Enum>(obj: &JsonValue, default: &'static str, enum_name: &'static str) -> E {
    let code = obj.as_str().unwrap_or(default);
    match E::from_code(code) {
        Some(val) => val,
        None => panic!("unknown {} representation: {}", enum_name, code),
    }
}

fn required_enum<E: Enum>(obj: &JsonValue, enum_name: &'static str) -> E {
    let code = obj.as_str().unwrap();
    match E::from_code(code) {
        Some(val) => val,
        None => panic!("unknown {} representation: {}", enum_name, code),
    }
}
