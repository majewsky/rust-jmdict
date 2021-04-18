/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::*;

//NOTE: Choose test words such that tests work with the `db-minimal` feature.
//We want the CI run to complete before we retire.

///Checks that glosses for the selected target languages are available.
#[test]
fn test_gloss_availability() {
    let entry = entries()
        .find(|e| e.kanji_elements().any(|k| k.text == "お母さん"))
        .unwrap();

    //while we're at it, test the decoding of entry numbers
    assert_eq!(entry.number, 1002650);

    let test_cases = &[
        ("eng", cfg!(feature = "translations-eng"), "mom"),
        (
            "dut",
            cfg!(feature = "translations-dut"),
            "moeder {honorifieke term}",
        ),
        ("fre", cfg!(feature = "translations-fre"), "mère"),
        ("ger", cfg!(feature = "translations-ger"), "Mama"),
        ("hun", cfg!(feature = "translations-hun"), "anya-"),
        ("rus", cfg!(feature = "translations-rus"), "мама, мамочка"),
        ("slv", cfg!(feature = "translations-slv"), "mati"),
        ("spa", cfg!(feature = "translations-spa"), "madre"),
        ("swe", cfg!(feature = "translations-swe"), "mamma"),
    ];

    for (lang_code, selected, gloss) in test_cases {
        let glosses: Vec<_> = entry
            .senses()
            .flat_map(|s| s.glosses())
            .filter(|g| g.language.code() == *lang_code)
            .map(|g| g.text)
            .collect();
        assert_eq!(
            *selected,
            !glosses.is_empty(),
            "language code was {}",
            *lang_code
        );

        if *selected {
            assert!(glosses.contains(gloss), "glosses were {:?}", glosses);
        }
    }
}

///Spot checks for correct decoding of priorities.
#[test]
fn test_priorities() {
    //Tests may be skipped if the test entry is not available, since entry
    //availability depends on the selection of target languages.
    if let Some((_, ke)) = find_by_keb("お参り") {
        assert_eq!(
            ke.priority,
            Priority {
                ichimango: PriorityInCorpus::Primary,
                news: PriorityInCorpus::Secondary,
                frequency_bucket: 36,
                ..Default::default()
            }
        );
    }

    if let Some((_, _, re)) = find_by_keb_reb("あの方", "あのかた") {
        assert_eq!(
            re.priority,
            Priority {
                additional: PriorityInCorpus::Primary,
                ..Default::default()
            }
        );
    }

    //`db-minimal` does not contain any gai1/gai2 vocabs
    #[cfg(not(feature = "db-minimal"))]
    {
        if let Some((_, re)) = find_by_reb("アーク") {
            assert_eq!(
                re.priority,
                Priority {
                    loanwords: PriorityInCorpus::Primary,
                    ..Default::default()
                }
            );
        }
    }
}

///Spot checks for correct decoding of enums.
#[test]
fn test_enums() {
    //Tests may be skipped if the test entry is not available, since entry
    //availability depends on the selection of target languages.

    //check for KanjiInfo
    if let Some((_, ke)) = find_by_keb("屹度") {
        assert_eq!(enum2str(ke.infos()), "Ateji");
    }

    //check for ReadingInfo (There are no entries with ReadingInfo in "db-minimal"
    //unless we include "scope-uncommon".)
    let (keb, reb, expected_infos) = if cfg!(feature = "db-minimal") {
        if cfg!(feature = "scope-uncommon") {
            ("彼処", "あしこ", "OutdatedKanaUsage")
        } else {
            ("", "", "")
        }
    } else {
        ("発条", "ばね", "GikunOrJukujikun")
    };
    if keb != "" {
        if let Some((_, _, re)) = find_by_keb_reb(keb, reb) {
            assert_eq!(enum2str(re.infos()), expected_infos);
        }
    }

    //All Sense lookups rely on a certain gloss, so we need to feature-gate on the gloss language.
    #[cfg(feature = "translations-eng")]
    {
        //check for PartOfSpeech
        let sense = find_sense("あっさり", "easily");
        assert_eq!(
            enum2str(sense.parts_of_speech()),
            "Adverb,AdverbTakingToParticle,SuruVerb"
        );

        //check for SenseTopic
        let sense = find_sense("御田", "oden");
        assert_eq!(enum2str(sense.topics()), "Food");

        //check for SenseInfo
        let sense = find_sense("うんこ", "poop");
        assert_eq!(enum2str(sense.infos()), "Colloquialism,ChildrensLanguage");

        //check for Dialect
        let sense = find_sense("ええ", "good");
        assert_eq!(enum2str(sense.dialects()), "Kansai");

        //check for GlossType
        let gloss_text = "in the time it takes to say \"ah!\"";
        let sense = find_sense("あっという間に", gloss_text);
        let gloss = sense.glosses().find(|g| g.text == gloss_text).unwrap();
        assert_eq!(gloss.gloss_type, GlossType::LiteralTranslation);
    }
}

///Spot checks for correct inclusion of various string fields.
#[test]
fn test_strings() {
    //All Sense lookups rely on a certain gloss, so we need to feature-gate on the gloss language.
    #[cfg(feature = "translations-eng")]
    {
        //check for stagk
        let (sense, expected_stagk) = if cfg!(feature = "db-minimal") {
            if cfg!(feature = "scope-uncommon") {
                (Some(find_sense("遇う", "to treat")), "遇う")
            } else {
                (None, "")
            }
        } else {
            (
                Some(find_sense("アンド", "AND (boolean operator)")),
                "ＡＮＤ",
            )
        };
        if let Some(sense) = sense {
            assert_eq!(strs2str(sense.applicable_kanji_elements()), expected_stagk);
        }

        //check for stagr
        let sense = find_sense("彼処", "genitals");
        assert_eq!(
            strs2str(sense.applicable_reading_elements()),
            "あそこ,あすこ,アソコ"
        );

        //check for xref
        let sense = find_sense("彼の", "the");
        assert_eq!(strs2str(sense.cross_references()), "どの,この・1,その・1");

        //check for ant (`db-minimal` has absolutely none of those)
        #[cfg(not(feature = "db-minimal"))]
        {
            let sense = find_sense("アンダー", "under");
            assert_eq!(strs2str(sense.antonyms()), "オーバー・2");
        }

        //check for s_inf
        let sense = find_sense("如何にも", "indeed");
        assert_eq!(
            strs2str(sense.freetext_infos()),
            "indicating emotive conviction"
        );
    }
}

///Spot checks for correct encoding of loanword sources.
#[test]
fn test_loanword_sources() {
    //All Sense lookups rely on a certain gloss, so we need to feature-gate on the gloss language.
    //Also, `db-minimal` has nearly no loanword sources to work with.
    #[cfg(all(feature = "translations-eng", not(feature = "db-minimal")))]
    {
        let sense = find_sense("アイメート", "seeing-eye dog");
        assert_eq!(
            &sense.loanword_sources().collect::<Vec<_>>(),
            &[LoanwordSource {
                text: "eye mate",
                language: "eng",
                is_partial: false,
                is_wasei: true,
            }]
        );

        //test with partial loanword sources
        #[cfg(feature = "scope-uncommon")]
        {
            let sense = find_sense("サブザック", "small knapsack");
            assert_eq!(
                &sense.loanword_sources().collect::<Vec<_>>(),
                &[
                    LoanwordSource {
                        text: "sub",
                        language: "eng",
                        is_partial: true,
                        is_wasei: true,
                    },
                    LoanwordSource {
                        text: "Sack",
                        language: "ger",
                        is_partial: true,
                        is_wasei: true,
                    }
                ]
            );
        }
    }
}

fn enum2str<E: Enum>(vals: impl Iterator<Item = E>) -> String {
    strs2str(vals.map(|v| v.constant_name()))
}

fn strs2str<'a>(vals: impl Iterator<Item = &'a str>) -> String {
    vals.enumerate()
        .map(|(i, v)| if i == 0 { v.into() } else { format!(",{}", v) })
        .collect()
}

fn find_by_keb(keb: &'static str) -> Option<(Entry, KanjiElement)> {
    let e = entries().find(|e| e.kanji_elements().any(|k| k.text == keb))?;
    Some((e, e.kanji_elements().find(|k| k.text == keb).unwrap()))
}

fn find_by_reb(reb: &'static str) -> Option<(Entry, ReadingElement)> {
    let e = entries().find(|e| e.reading_elements().any(|r| r.text == reb))?;
    Some((e, e.reading_elements().find(|r| r.text == reb).unwrap()))
}

fn find_by_keb_reb(
    keb: &'static str,
    reb: &'static str,
) -> Option<(Entry, KanjiElement, ReadingElement)> {
    let e = entries().find(|e| e.kanji_elements().any(|k| k.text == keb))?;
    let ke = e.kanji_elements().find(|k| k.text == keb).unwrap();
    let re = e.reading_elements().find(|r| r.text == reb)?;
    Some((e, ke, re))
}

fn find_sense(jp_text: &'static str, gloss: &'static str) -> Sense {
    entries()
        .find(|e| {
            (e.kanji_elements().any(|k| k.text == jp_text)
                || e.reading_elements().any(|r| r.text == jp_text))
                && e.senses().any(|s| s.glosses().any(|g| g.text == gloss))
        })
        .unwrap()
        .senses()
        .find(|s| s.glosses().any(|g| g.text == gloss))
        .unwrap()
}
