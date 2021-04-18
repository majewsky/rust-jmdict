/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use std::fmt::Debug;

#[test]
fn check_consistency() {
    //This test runs through the data files in the repository a second time and checks that
    //entries() contains exactly what we want. This test especially verifies that all indexes into
    //omniarrays are within bounds and point to the right stuff.
    struct Visitor(crate::Entries);

    impl jmdict_traverse::Visitor for Visitor {
        fn process_entry(&mut self, entry: &jmdict_traverse::RawEntry) {
            match self.0.next() {
                None => panic!("jmdict::entries() exhausted before end of traversal"),
                Some(actual) => entry.check(&actual),
            };
        }
    }

    let opts = jmdict_traverse::Options {
        is_db_minimal: cfg!(feature = "db-minimal"),
        with_uncommon: cfg!(feature = "scope-uncommon"),
        with_archaic: cfg!(feature = "scope-archaic"),
    };

    let mut v = Visitor(crate::entries());
    jmdict_traverse::process_dictionary(&mut v, opts);
    assert!(v.0.next().is_none(), "not all entries were exhausted");
}

trait Check<A> {
    fn check(&self, actual: &A);
}

fn check_vec<A, E: Check<A>>(
    expected: &Vec<E>,
    actual: impl Iterator<Item = A> + ExactSizeIterator,
) {
    assert_eq!(expected.len(), actual.len());
    for (expected, actual) in expected.iter().zip(actual) {
        expected.check(&actual);
    }
}

impl<E: Debug + PartialEq<A>, A: Debug + PartialEq<E>> Check<A> for E {
    fn check(&self, actual: &A) {
        assert_eq!(self, actual);
    }
}

impl Check<crate::Entry> for jmdict_traverse::RawEntry<'_> {
    fn check(&self, actual: &crate::Entry) {
        let expected = self;
        check_vec(&expected.k_ele, actual.kanji_elements());
        check_vec(&expected.r_ele, actual.reading_elements());
        check_vec(&expected.sense, actual.senses());
    }
}

impl Check<crate::KanjiElement> for jmdict_traverse::RawKanjiElement<'_> {
    fn check(&self, actual: &crate::KanjiElement) {
        let expected = self;
        assert_eq!(expected.keb, actual.text);
        check_vec(&expected.ke_inf, actual.infos());
    }
}

impl Check<crate::ReadingElement> for jmdict_traverse::RawReadingElement<'_> {
    fn check(&self, actual: &crate::ReadingElement) {
        let expected = self;
        assert_eq!(expected.reb, actual.text);
        check_vec(&expected.re_inf, actual.infos());
    }
}

impl Check<crate::Sense> for jmdict_traverse::RawSense<'_> {
    fn check(&self, actual: &crate::Sense) {
        let expected = self;
        check_vec(&expected.stagk, actual.applicable_kanji_elements());
        check_vec(&expected.stagr, actual.applicable_reading_elements());
        check_vec(&expected.pos, actual.parts_of_speech());
        check_vec(&expected.xref, actual.cross_references());
        check_vec(&expected.ant, actual.antonyms());
        check_vec(&expected.field, actual.topics());
        check_vec(&expected.misc, actual.infos());
        check_vec(&expected.s_inf, actual.freetext_infos());
        check_vec(&expected.lsource, actual.loanword_sources());
        check_vec(&expected.dial, actual.dialects());
        check_vec(&expected.gloss, actual.glosses());
    }
}

impl Check<crate::LoanwordSource> for jmdict_traverse::RawLSource<'_> {
    fn check(&self, actual: &crate::LoanwordSource) {
        let expected = self;
        assert_eq!(expected.lang, actual.language);
        assert_eq!(expected.text, actual.text);
        assert_eq!(expected.is_partial, actual.is_partial);
        assert_eq!(expected.is_wasei, actual.is_wasei);
    }
}

impl Check<crate::Gloss> for jmdict_traverse::RawGloss<'_> {
    fn check(&self, actual: &crate::Gloss) {
        let expected = self;
        assert_eq!(expected.lang, actual.language);
        assert_eq!(expected.text, actual.text);
        assert_eq!(expected.g_type, actual.gloss_type);
    }
}
