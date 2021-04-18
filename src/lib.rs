/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

//! The [JMdict file](https://www.edrdg.org/jmdict/j_jmdict.html) is a comprehensive multilingual
//! dictionary of the Japanese language. The original JMdict file, included in this repository (and
//! hence, in releases of this crate) comes as XML. Instead of stuffing the XML in the binary
//! directly, this crate parses the XML at compile-time and generates an optimized representation
//! that is compiled into the binary. The crate's API affords type-safe access to this embedded
//! database.
//!
//! # WARNING: Licensing on database files
//!
//! The database files compiled into the crate are licensed from the Electronic Dictionary Research
//! and Development Group under Creative Commons licenses. Applications linking this crate directly
//! oder indirectly must display appropriate copyright notices to users. Please refer to the
//! [EDRDG's license statement](https://www.edrdg.org/edrdg/licence.html) for details.
//!
//! # Basic usage
//!
//! The database is accessed through the [entries() function](entries) which provides an iterator
//! over all database entries compiled into the application. While traversing the database and its
//! entries, you will find that, whenever you expect a list of something, you will get an iterator
//! instead. These iterators provide an abstraction between you as the user of the library, and the
//! physical representation of the database as embedded in the binary.
//!
//! The following example looks up the reading for お母さん in the database:
//!
//! ```
//! let kanji_form = "お母さん";
//!
//! let entry = jmdict::entries().find(|e| {
//!     e.kanji_elements().any(|k| k.text == kanji_form)
//! }).unwrap();
//!
//! let reading_form = entry.reading_elements().next().unwrap().text;
//! assert_eq!(reading_form, "おかあさん");
//! ```
//!
//! # Cargo features
//!
//! ### Common configurations
//!
//! * The `default` feature includes the most common words (about 30000 entries) and only their
//!   English translations.
//! * The `full` feature includes everything in the JMdict.
//!
//! ### Entry selection
//!
//! * The `scope-uncommon` feature includes uncommon words and glosses.
//! * The `scope-archaic` feature includes glosses with the "archaic" label. If disabled, the
//!   [PartOfSpeech] enum will not include variants that are only relevant for archaic vocabulary,
//!   such as obsolete conjugation patterns. (The [AllPartOfSpeech] enum always contains all
//!   variants.)
//!
//! ### Target languages
//!
//! At least one target language must be selected. Selecting a target language will include all
//! available translations in that language. Entries that do not have any translation in any of the
//! selected languages will be skipped.
//!
//! * `translations-eng`: English (included in `default`)
//! * `translations-dut`: Dutch
//! * `translations-fre`: French
//! * `translations-ger`: German
//! * `translations-hun`: Hungarian
//! * `translations-rus`: Russian
//! * `translations-slv`: Slovenian
//! * `translations-spa`: Spanish
//! * `translations-swe`: Swedish
//!
//! The [GlossLanguage] enum will only contain variants corresponding to the enabled target
//! languages. For example, in the default configuration, `GlossLanguage::English` will be the only
//! variant. (The [AllGlossLanguage] enum always contains all variants.)
//!
//! ### Crippled builds: `db-minimal`
//!
//! When the `db-minimal` feature is enabled, only a severly reduced portion of the JMdict will
//! be parsed (to be exact, only chunks 000, 100 and 999). This is also completely useless for
//! actual usage, but allows for quick edit-compile-test cycles while working on this crate's
//! code.
//!
//! ### Crippled builds: `db-empty`
//!
//! When the `db-empty` feature is enabled, downloading and parsing of the JMdict contents is
//! disabled entirely. The crate is compiled as usual, but `entries()` will be an empty list.
//! This is useful for documentation builds like for `docs.rs`, where `--all-features` is given.

pub use jmdict_enums::{
    AllGlossLanguage, AllPartOfSpeech, Dialect, DisabledVariant, Enum, GlossLanguage, GlossType,
    KanjiInfo, PartOfSpeech, Priority, PriorityInCorpus, ReadingInfo, SenseInfo, SenseTopic,
};
mod payload;
use payload::*;

#[cfg(test)]
mod test_consistency;
#[cfg(test)]
mod test_feature_matrix;
#[cfg(test)]
mod test_ordering;

///Returns an iterator over all entries in the database.
pub fn entries() -> Entries {
    Entries::new()
}

///An entry in the JMdict dictionary.
///
///Each entry has zero or more [kanji elements](KanjiElement), one or more
///[reading elements](ReadingElement) and one or more [senses](Sense). Elements contain the
///Japanese representation of the vocabulary or phrase. Whereas reading elements consist of only
///kana, kanji elements will contain characters from non-kana scripts, most commonly kanji. Senses
///contain the translation of the vocabulary or phrase in other languages, most commonly English.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    ///The sequence number for this Entry as it appears in the JMdict. Numbers start around 1000000
    ///and typically increment in steps of 5 or 10. (It's like BASIC line numbers, if you're old
    ///enough to understand that reference.) The [Entries] iterator guarantees entries to appear
    ///ordered by sequence number.
    pub number: u32,
    kanji_elements_iter: KanjiElements,
    reading_elements_iter: ReadingElements,
    senses_iter: Senses,
}

impl Entry {
    pub fn kanji_elements(&self) -> KanjiElements {
        self.kanji_elements_iter
    }

    pub fn reading_elements(&self) -> ReadingElements {
        self.reading_elements_iter
    }

    pub fn senses(&self) -> Senses {
        self.senses_iter
    }
}

///A representation of a dictionary entry using kanji or other non-kana scripts.
///
///Each [Entry] may have any number of these (including none). For each kanji element, the entry
///will also have [reading elements](ReadingElement) to indicate how to read this kanji element.
#[derive(Clone, Copy, Debug)]
pub struct KanjiElement {
    pub text: &'static str,
    pub priority: Priority,
    info_iter: KanjiInfos,
}

impl KanjiElement {
    pub fn infos(&self) -> KanjiInfos {
        self.info_iter
    }
}

///A representation of a dictionary entry using only kana.
///
///Each [Entry] will have zero or more of these. When an entry has both kanji elements and reading
///elements, the kana usage will be consistent between them, that is: If the kanji element contains
///katakana, there is also a corresponding reading element that contains katakana as well.
#[derive(Clone, Copy, Debug)]
pub struct ReadingElement {
    pub text: &'static str,
    pub priority: Priority,
    info_iter: ReadingInfos,
}

impl ReadingElement {
    pub fn infos(&self) -> ReadingInfos {
        self.info_iter
    }
}

///The translational equivalent of a Japanese word or phrase.
///
///Where there are several distinctly different meanings of the word, its [Entry] will have
///multiple senses. Each particular translation is a [Gloss], of which there may be multiple within
///a single sense.
///
///For instance, the entry for 折角 contains one sense with the glosses "with trouble" and "at
///great pains". Those glosses all represent the same meaning, so they appear in one sense. There
///is also a sense with the glosses "rare", "precious", "valuable" and "long-awaited". Those
///glosses represent a different meaning from "with trouble" or "at great pains", so they appear in
///a separate sense. (And in fact, 折角 has even more senses.)
#[derive(Clone, Copy, Debug)]
pub struct Sense {
    stagk_iter: Strings,
    stagr_iter: Strings,
    pos_iter: PartsOfSpeech,
    cross_refs_iter: Strings,
    antonyms_iter: Strings,
    topics_iter: SenseTopics,
    info_iter: SenseInfos,
    freetext_info_iter: Strings,
    loanword_sources_iter: LoanwordSources,
    dialects_iter: Dialects,
    glosses_iter: Glosses,
}

impl Sense {
    ///If not empty, this sense only applies to these [KanjiElements] out of all the
    ///[KanjiElements] in this [Entry].
    pub fn applicable_kanji_elements(&self) -> Strings {
        self.stagk_iter
    }

    ///If not empty, this sense only applies to these [ReadingElements] out of all the
    ///[ReadingElements] in this [Entry].
    pub fn applicable_reading_elements(&self) -> Strings {
        self.stagr_iter
    }

    pub fn parts_of_speech(&self) -> PartsOfSpeech {
        self.pos_iter
    }

    ///If not empty, contains the text of [KanjiElements] or [ReadingElements] of other [Entries]
    ///with a similar meaning or sense. In some cases, a [KanjiElement]'s text will be followed by
    ///a [Reading Element]'s text and/or a sense number to provide a precise target for the
    ///cross-reference. Where this happens, a katakana middle dot (`・`, U+30FB) is placed between
    ///the components of the cross-reference.
    ///
    ///TODO: Provide a structured type for these kinds of references.
    pub fn cross_references(&self) -> Strings {
        self.cross_refs_iter
    }

    ///If not empty, contains the text of [KanjiElements] or [ReadingElements] of other [Entries]
    ///which are antonyms of this sense.
    pub fn antonyms(&self) -> Strings {
        self.antonyms_iter
    }

    pub fn topics(&self) -> SenseTopics {
        self.topics_iter
    }

    pub fn infos(&self) -> SenseInfos {
        self.info_iter
    }

    ///If not empty, contains additional information about this sence (e.g. level of currency or
    ///other nuances) that cannot be expressed by the other, more structured fields.
    pub fn freetext_infos(&self) -> Strings {
        self.freetext_info_iter
    }

    ///If not empty, contains source words in other languages from which this vocabulary has been
    ///borrowed in this sense.
    pub fn loanword_sources(&self) -> LoanwordSources {
        self.loanword_sources_iter
    }

    ///If not empty, this [Sense] of the [Entry] only appears in the given [Dialects] of Japanese.
    pub fn dialects(&self) -> Dialects {
        self.dialects_iter
    }

    pub fn glosses(&self) -> Glosses {
        self.glosses_iter
    }
}

///A source word in other language which a particular [Sense] of an [Entry] has been borrowed from.
///
///There may be multiple sources for a single [Sense] when it is not clear from which language a
///word has been borrowed (e.g. "セレナーデ" lists both the French word "sérénade" and the German
///word "Serenade" as loanword sources), or if the vocabulary is a composite word with multiple
///distinct sources (e.g. "サブリュック" is a combination of the English prefix "sub-" and the
///German word "Rucksack").
///
///Within an [Entry], glosses appear in the [Sense].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoanwordSource {
    pub text: &'static str,
    ///The [ISO 639-2/B code](https://en.wikipedia.org/wiki/List_of_ISO_639-2_codes) for the
    ///language from which the word was borrowed, e.g. "ger" for German or "chi" for Chinese.
    pub language: &'static str,
    ///Whether this source applies only to part of the loanword. Note that this flag is not always
    ///present in the JMdict when it should be.
    pub is_partial: bool,
    ///Whether this loanword is a [Wasei-eigo](https://en.wikipedia.org/wiki/Wasei-eigo).
    pub is_wasei: bool,
}

///A particular translation or explanation for a Japanese word or phrase in a different language.
///
///Within an [Entry], glosses appear in the [Sense].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gloss {
    pub language: GlossLanguage,
    pub text: &'static str,
    pub gloss_type: GlossType,
}

///We cannot do `pub type KanjiElements = Range<KanjiElement, N>` etc. because Range<T, N> is
///private to the crate, so instead we declare a bunch of iterator types that wrap Range<T, N>.
macro_rules! wrap_iterator {
    ($val: ty, $size: literal, $iter: ident) => {
        ///An iterator providing fast access to objects in the database. Instances of this iterator
        ///can be copied cheaply.
        #[derive(Clone, Copy, Debug)]
        pub struct $iter(Range<$val, $size>);

        impl From<Range<$val, $size>> for $iter {
            fn from(r: Range<$val, $size>) -> $iter {
                $iter(r)
            }
        }

        impl std::iter::Iterator for $iter {
            type Item = $val;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }
        }

        impl std::iter::ExactSizeIterator for $iter {
            fn len(&self) -> usize {
                self.0.len()
            }
        }
    };
}

wrap_iterator!(KanjiElement, 5, KanjiElements);
wrap_iterator!(KanjiInfo, 1, KanjiInfos);
wrap_iterator!(ReadingElement, 5, ReadingElements);
wrap_iterator!(ReadingInfo, 1, ReadingInfos);
wrap_iterator!(Sense, 5, Senses);
wrap_iterator!(&'static str, 2, Strings);
wrap_iterator!(PartOfSpeech, 1, PartsOfSpeech);
wrap_iterator!(SenseTopic, 1, SenseTopics);
wrap_iterator!(SenseInfo, 1, SenseInfos);
wrap_iterator!(LoanwordSource, 4, LoanwordSources);
wrap_iterator!(Dialect, 1, Dialects);
wrap_iterator!(Gloss, 2, Glosses);

///An iterator providing fast access to objects in the database. Instances of this iterator
///can be copied cheaply.
#[derive(Clone, Copy)]
pub struct Entries {
    //This iterator is very similar to Range<T, N>, but cannot be implemented in terms of it
    //because it iterates over ALL_ENTRY_OFFSETS instead of ALL_DATA.
    start: usize,
    end: usize,
}

impl Entries {
    fn new() -> Self {
        Self {
            start: 0,
            end: entry_count(),
        }
    }
}

impl std::iter::Iterator for Entries {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let entry = get_entry(self.start);
            self.start += 1;
            Some(entry)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.end - self.start;
        (count, Some(count))
    }
}

impl std::iter::ExactSizeIterator for Entries {
    fn len(&self) -> usize {
        self.end - self.start
    }
}
