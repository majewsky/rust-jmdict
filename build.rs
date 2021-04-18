/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

#[cfg(not(any(
    feature = "translations-eng",
    feature = "translations-dut",
    feature = "translations-fre",
    feature = "translations-ger",
    feature = "translations-hun",
    feature = "translations-rus",
    feature = "translations-slv",
    feature = "translations-spa",
    feature = "translations-swe"
)))]
compile_error!("no target languages selected (select at least one \"translations-XXX\" feature)");

use jmdict_enums::*;
use std::convert::TryInto;
use std::io::Write;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let opts = jmdict_traverse::Options {
        is_db_minimal: cfg!(feature = "db-minimal"),
        with_uncommon: cfg!(feature = "scope-uncommon"),
        with_archaic: cfg!(feature = "scope-archaic"),
    };

    let mut omni: OmniBuffer = Default::default();
    if cfg!(not(feature = "db-empty")) {
        jmdict_traverse::process_dictionary(&mut omni, opts);
    }

    write_u32s(&path_to("entry_offsets.dat"), &omni.entry_offsets);
    write_u32s(&path_to("payload.dat"), &omni.data);
    std::fs::write(&path_to("strings.txt"), &omni.text).unwrap();
}

fn path_to(filename: &str) -> std::path::PathBuf {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    std::path::Path::new(&out_dir).join(filename)
}

fn write_u32s(path: &std::path::Path, vals: &[u32]) {
    let f = std::fs::File::create(&path).unwrap();
    let mut f = std::io::BufWriter::new(f);
    for val in vals {
        f.write_all(&val.to_ne_bytes()).unwrap();
    }
}

///Helper type for references into OmniBuffer::data or OmniBuffer::text.
///Gets constructed as `(start, end).into()` in the respective OmniBuffer methods.
struct StoredRef {
    start: u32,
    end: u32,
}

impl From<(usize, usize)> for StoredRef {
    fn from(val: (usize, usize)) -> Self {
        let (start, end) = val;
        let start = start.try_into().unwrap();
        let end = end.try_into().unwrap();
        Self { start, end }
    }
}

///Buffer where all payload gets accumulated before being written into the generated data files.
///Check the explanations in CONTRIBUTING.md for how this works, and why it was built this way.
#[derive(Default)]
struct OmniBuffer {
    entry_offsets: Vec<u32>,
    data: Vec<u32>,
    text: String,
}

impl OmniBuffer {
    pub fn push_str(&mut self, text: &str) -> StoredRef {
        //optimization: empty text doesn't require any work
        if text.is_empty() {
            return (0, 0).into();
        }

        let start = self.text.len();
        self.text.push_str(text);
        let end = self.text.len();
        (start, end).into()
    }

    pub fn push_data(&mut self, data: &[u32]) -> StoredRef {
        //optimization: empty arrays don't require any work
        if data.is_empty() {
            return (0, 0).into();
        }

        let start = self.data.len();
        self.data.extend(data);
        (start, start + data.len()).into()
    }

    pub fn push_array<T: ToPayload>(&mut self, data: &[T]) -> StoredRef {
        //optimization: empty arrays don't require any work
        if data.is_empty() {
            return (0, 0).into();
        }

        //render all items into a contiguous Vec<u32>
        let size = T::size();
        let mut repr = vec![0u32; data.len() * size];
        for (idx, elem) in data.iter().enumerate() {
            elem.encode_one(self, &mut repr[(idx * size)..((idx + 1) * size)]);
        }

        self.push_data(&repr)
    }
}

impl jmdict_traverse::Visitor for OmniBuffer {
    fn notify_data_file_path(&mut self, path: &str) {
        println!("cargo:rerun-if-changed={}", &path);
    }

    fn process_entry(&mut self, entry: &jmdict_traverse::RawEntry) {
        let size = jmdict_traverse::RawEntry::size();
        let mut repr = vec![0u32; size];
        entry.encode_one(self, &mut repr);
        let r = self.push_data(&repr);
        self.entry_offsets.push(r.start);
    }
}

//Like omni.push_array(), but does not push the resulting array just yet.
fn push_array<T: ToPayload>(buf: &mut Vec<u32>, omni: &mut OmniBuffer, array: &[T]) -> u32 {
    if !array.is_empty() {
        let size = T::size();
        let mut repr = vec![0u32; array.len() * size];
        for (idx, elem) in array.iter().enumerate() {
            elem.encode_one(omni, &mut repr[(idx * size)..((idx + 1) * size)]);
        }
        buf.extend(repr);
    }

    buf.len() as u32
}

///Helper trait for encoding types from the jmdict-traverse crate into a sequence of u32 for
///embedding in OmniBuffer::data.
trait ToPayload {
    ///How many u32 are needed to encode one item of this type.
    fn size() -> usize;

    ///Encode one item of this type into the given preallocated buffer of length `Self::size()`.
    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]);
}

//NOTE: It would be really nice to just do `impl ToPayload for T where T: EnumPayload`, but this
//conflicts with all other `impl ToPayload` under the current specialization rules.
macro_rules! enum_to_payload {
    ($t:ident) => {
        impl ToPayload for $t {
            fn size() -> usize {
                1
            }

            fn encode_one(&self, _omni: &mut OmniBuffer, buf: &mut [u32]) {
                buf[0] = self.to_u32();
            }
        }
    };
}

enum_to_payload!(KanjiInfo);
enum_to_payload!(ReadingInfo);
enum_to_payload!(PartOfSpeech);
enum_to_payload!(SenseTopic);
enum_to_payload!(SenseInfo);
enum_to_payload!(Dialect);

impl ToPayload for jmdict_traverse::RawEntry<'_> {
    fn size() -> usize {
        4
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        //Instead of using `omni.push_array()` on each member and encoding each StoredRef
        //separately, we concatenate the payload representations of all member arrays and
        //`push_data()` them all at once. We then encode that StoredRef, plus offsets to split the
        //encoded array back into its constituents. Since each encoded array is rather short, the
        //offsets fit into a single byte, so we can encode both (plus self.ent_seq) in a single u32.
        //
        //Compared to the naive layout as 3 StoredRef + 1 u32 (28 bytes), we save 12 bytes per Sense.

        let mut dbuf = Vec::new();
        let offset1 = push_array(&mut dbuf, omni, &self.k_ele);
        let offset2 = push_array(&mut dbuf, omni, &self.r_ele);
        push_array(&mut dbuf, omni, &self.sense);

        let r = omni.push_data(&dbuf);
        buf[0] = r.start;
        buf[1] = r.end;
        buf[2] = offset1 + (offset2 << 16);
        buf[3] = self.ent_seq;
    }
}

impl ToPayload for jmdict_traverse::RawKanjiElement<'_> {
    fn size() -> usize {
        5
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        buf[0] = self.ke_pri.to_u32();
        let r = omni.push_str(self.keb);
        buf[1] = r.start;
        buf[2] = r.end;
        let r = omni.push_array(&self.ke_inf);
        buf[3] = r.start;
        buf[4] = r.end;
    }
}

impl ToPayload for jmdict_traverse::RawReadingElement<'_> {
    fn size() -> usize {
        5
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        buf[0] = self.re_pri.to_u32();
        let r = omni.push_str(self.reb);
        buf[1] = r.start;
        buf[2] = r.end;
        let r = omni.push_array(&self.re_inf);
        buf[3] = r.start;
        buf[4] = r.end;
    }
}

impl ToPayload for jmdict_traverse::RawSense<'_> {
    fn size() -> usize {
        5
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        //Instead of using `omni.push_array()` on each member and encoding each StoredRef
        //separately, we concatenate the payload representations of all member arrays and
        //`push_data()` them all at once. We then encode that StoredRef, plus offsets to split the
        //encoded array back into its constituents. Since each encoded array is rather short, the
        //offsets fit into a single byte, so we can encode four at a time in a single u32.
        //
        //Compared to the naive layout as 11 StoredRef (88 bytes), we save 68 bytes per Sense.

        let mut dbuf = Vec::new();
        let offset1 = push_array(&mut dbuf, omni, &self.stagk);
        let offset2 = push_array(&mut dbuf, omni, &self.stagr);
        let offset3 = push_array(&mut dbuf, omni, &self.pos);
        let offset4 = push_array(&mut dbuf, omni, &self.xref);
        let offset5 = push_array(&mut dbuf, omni, &self.ant);
        let offset6 = push_array(&mut dbuf, omni, &self.field);
        let offset7 = push_array(&mut dbuf, omni, &self.misc);
        let offset8 = push_array(&mut dbuf, omni, &self.s_inf);
        let offset9 = push_array(&mut dbuf, omni, &self.lsource);
        let offset10 = push_array(&mut dbuf, omni, &self.dial);
        push_array(&mut dbuf, omni, &self.gloss);

        let r = omni.push_data(&dbuf);
        buf[0] = r.start;
        buf[1] = r.end;
        buf[2] = offset1 + (offset2 << 8) + (offset3 << 16) + (offset4 << 24);
        buf[3] = offset5 + (offset6 << 8) + (offset7 << 16) + (offset8 << 24);
        buf[4] = offset9 + (offset10 << 8);
    }
}

impl ToPayload for jmdict_traverse::RawLSource<'_> {
    fn size() -> usize {
        4
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        let r = omni.push_str(self.text);
        buf[0] = r.start;
        buf[1] = r.end;
        let r = omni.push_str(self.lang);
        buf[2] = r.start;
        buf[3] = r.end;
        //`omni.text` is significantly shorter than 2^28 bytes, so we can shove those two booleans
        //into the highest bits of one of the offset values
        if self.is_partial {
            buf[0] |= 0x10000000;
        }
        if self.is_wasei {
            buf[0] |= 0x20000000;
        }
    }
}

impl ToPayload for jmdict_traverse::RawGloss<'_> {
    fn size() -> usize {
        2
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        //`omni.text` is never larger than 30-40 MiB. That's slightly more than 2^24 bytes, but
        //comfortably below 2^28 bytes. We can therefore use the upper 4 bits of `buf[0]` and
        //`buf[1]`, respectively, to encode `self.lang` and `self.g_type`.
        let r = omni.push_str(self.text);
        buf[0] = r.start | (self.lang.to_u32() << 28);
        buf[1] = r.end | (self.g_type.to_u32() << 28);
    }
}

impl<'a> ToPayload for &'a str {
    fn size() -> usize {
        2
    }

    fn encode_one(&self, omni: &mut OmniBuffer, buf: &mut [u32]) {
        let r = omni.push_str(self);
        buf[0] = r.start;
        buf[1] = r.end;
    }
}

impl ToPayload for u32 {
    fn size() -> usize {
        1
    }

    fn encode_one(&self, _omni: &mut OmniBuffer, buf: &mut [u32]) {
        buf[0] = *self;
    }
}
