/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

//! This file contains the type definitions for the database payload. Because we want the payload
//! format to be an implementation detail, the entire module is private and hence these types are
//! not part of the public API.

use crate::*;
use std::convert::TryInto;
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////
// generic machinery for iterating over ALL_DATA

pub(crate) trait FromPayload<const N: usize> {
    ///Given `&ALL_DATA[offset..]`, unmarshals the data starting from that offset into a value of
    ///self. Returns the unmarshaled value, as well as the amount of u32 that were consumed.
    fn get(data: &[u32; N]) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Range<T: FromPayload<N>, const N: usize> {
    pub start: usize,
    pub end: usize,
    pub phantom: PhantomData<T>,
}

impl<T: FromPayload<N>, const N: usize> Range<T, N> {
    pub(crate) fn new(start: u32, end: u32) -> Self {
        Self {
            start: start.try_into().unwrap(),
            end: end.try_into().unwrap(),
            phantom: PhantomData,
        }
    }
}

impl<T: FromPayload<N>, const N: usize> std::iter::Iterator for Range<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let data = &ALL_DATA[self.start..(self.start + N)];
            let item = T::get(data.try_into().unwrap());
            self.start += N;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = (self.end - self.start) / N;
        (count, Some(count))
    }
}

impl<T: FromPayload<N>, const N: usize> std::iter::ExactSizeIterator for Range<T, N> {
    fn len(&self) -> usize {
        (self.end - self.start) / N
    }
}

////////////////////////////////////////////////////////////////////////////////
// concrete types

pub(crate) fn entry_count() -> usize {
    ALL_ENTRY_OFFSETS.len()
}

pub(crate) fn get_entry(idx: usize) -> Entry {
    let offset: usize = ALL_ENTRY_OFFSETS[idx].try_into().unwrap();
    let data = &ALL_DATA[offset..(offset + 4)];

    let (start, end) = (data[0], data[1]);
    let mid1 = start + (data[2] & 0x0000FFFF);
    let mid2 = start + ((data[2] & 0xFFFF0000) >> 16);

    Entry {
        number: data[3],
        kanji_elements_iter: Range::new(start, mid1).into(),
        reading_elements_iter: Range::new(mid1, mid2).into(),
        senses_iter: Range::new(mid2, end).into(),
    }
}

impl FromPayload<5> for KanjiElement {
    fn get(data: &[u32; 5]) -> Self {
        Self {
            priority: jmdict_enums::EnumPayload::from_u32(data[0]),
            text: get_str(data[1], data[2]),
            info_iter: Range::new(data[3], data[4]).into(),
        }
    }
}

impl FromPayload<1> for KanjiInfo {
    fn get(data: &[u32; 1]) -> Self {
        jmdict_enums::EnumPayload::from_u32(data[0])
    }
}

impl FromPayload<5> for ReadingElement {
    fn get(data: &[u32; 5]) -> Self {
        Self {
            priority: jmdict_enums::EnumPayload::from_u32(data[0]),
            text: get_str(data[1], data[2]),
            info_iter: Range::new(data[3], data[4]).into(),
        }
    }
}

impl FromPayload<1> for ReadingInfo {
    fn get(data: &[u32; 1]) -> Self {
        jmdict_enums::EnumPayload::from_u32(data[0])
    }
}

impl FromPayload<5> for Sense {
    fn get(data: &[u32; 5]) -> Self {
        let (start, end) = (data[0], data[1]);
        let mid1 = start + (data[2] & 0x000000FF);
        let mid2 = start + ((data[2] & 0x0000FF00) >> 8);
        let mid3 = start + ((data[2] & 0x00FF0000) >> 16);
        let mid4 = start + ((data[2] & 0xFF000000) >> 24);
        let mid5 = start + (data[3] & 0x000000FF);
        let mid6 = start + ((data[3] & 0x0000FF00) >> 8);
        let mid7 = start + ((data[3] & 0x00FF0000) >> 16);
        let mid8 = start + ((data[3] & 0xFF000000) >> 24);
        let mid9 = start + (data[4] & 0x000000FF);
        let mid10 = start + ((data[4] & 0x0000FF00) >> 8);

        Self {
            stagk_iter: Range::new(start, mid1).into(),
            stagr_iter: Range::new(mid1, mid2).into(),
            pos_iter: Range::new(mid2, mid3).into(),
            cross_refs_iter: Range::new(mid3, mid4).into(),
            antonyms_iter: Range::new(mid4, mid5).into(),
            topics_iter: Range::new(mid5, mid6).into(),
            info_iter: Range::new(mid6, mid7).into(),
            freetext_info_iter: Range::new(mid7, mid8).into(),
            loanword_sources_iter: Range::new(mid8, mid9).into(),
            dialects_iter: Range::new(mid9, mid10).into(),
            glosses_iter: Range::new(mid10, end).into(),
        }
    }
}

impl FromPayload<1> for PartOfSpeech {
    fn get(data: &[u32; 1]) -> Self {
        jmdict_enums::EnumPayload::from_u32(data[0])
    }
}

impl FromPayload<1> for SenseTopic {
    fn get(data: &[u32; 1]) -> Self {
        jmdict_enums::EnumPayload::from_u32(data[0])
    }
}

impl FromPayload<1> for SenseInfo {
    fn get(data: &[u32; 1]) -> Self {
        jmdict_enums::EnumPayload::from_u32(data[0])
    }
}

impl FromPayload<4> for LoanwordSource {
    fn get(data: &[u32; 4]) -> Self {
        Self {
            text: get_str(data[0] & 0x0FFFFFFF, data[1]),
            language: get_str(data[2], data[3]),
            is_partial: (data[0] & 0x10000000) == 0x10000000,
            is_wasei: (data[0] & 0x20000000) == 0x20000000,
        }
    }
}

impl FromPayload<1> for Dialect {
    fn get(data: &[u32; 1]) -> Self {
        jmdict_enums::EnumPayload::from_u32(data[0])
    }
}

impl FromPayload<2> for Gloss {
    fn get(data: &[u32; 2]) -> Self {
        let lang_code = (data[0] & 0xF0000000) >> 28;
        let type_code = (data[1] & 0xF0000000) >> 28;
        Gloss {
            text: get_str(data[0] & 0x0FFFFFFF, data[1] & 0x0FFFFFFF),
            language: jmdict_enums::EnumPayload::from_u32(lang_code),
            gloss_type: jmdict_enums::EnumPayload::from_u32(type_code),
        }
    }
}

impl FromPayload<2> for &'static str {
    fn get(data: &[u32; 2]) -> Self {
        get_str(data[0], data[1])
    }
}

fn get_str(start: u32, end: u32) -> &'static str {
    let start = start.try_into().unwrap();
    let end = end.try_into().unwrap();
    &ALL_TEXTS[start..end]
}

////////////////////////////////////////////////////////////////////////////////
// embedded data

//NOTE: We would only need 4-byte alignment, but 16-byte is the smallest alignment interval that
//the align_data crate offers.

use align_data::{include_aligned, Align16};

const fn as_u32_slice(input: &'static [u8]) -> &'static [u32] {
    unsafe {
        let ptr = input.as_ptr() as *const u32;
        std::slice::from_raw_parts(ptr, input.len() / 4)
    }
}

static ALL_ENTRY_OFFSETS: &[u32] =
    as_u32_slice(include_aligned!(Align16, concat!(env!("OUT_DIR"), "/entry_offsets.dat")));
static ALL_DATA: &[u32] = as_u32_slice(include_aligned!(Align16, concat!(env!("OUT_DIR"), "/payload.dat")));
static ALL_TEXTS: &str = include_str!(concat!(env!("OUT_DIR"), "/strings.txt"));
