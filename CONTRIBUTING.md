# Contributing

Issues and PRs are welcome, but I cannot give any specific guarantees how fast I'll get to them. Some specific remarks:

* If the copy of JMdict is outdated and you need a newer version, open an issue and I'll make a new release with a newer
  copy for you. Please don't send a PR for this; I have no way to verify the diff and will do the import myself anyway.
* When making changes, please also add entries to `CHANGELOG.md` to describe your change, in the topmost "Unreleased"
  section. (If a release was recently made and there is no "Unreleased" section, start one.)

# Explanations

## Payload structure

The obvious idea would be to have `build.rs` generate a bunch of code like this...

```rust
// many fields elided for brevity
static ENTRIES: &[Entry] = &[
  Entry {
    sequence_number: 1000150,
    kanji_elements: &[
      KanjiElement {
        text: "ＲＳ２３２ケーブル",
      },
    ],
    reading_elements: &[
      ReadingElement {
        text: "アールエスにさんにケーブル",
      },
    ],
    senses: &[
      Sense {
        parts_of_speech: &[
          PartOfSpeech::Noun,
        ],
        glosses: &[
          Gloss {
            text: "rs232 cable",
          },
        ],
      },
    ],
  },
  ...
];
```

...and just `include!()` it into the main binary. The problem with this is that each `&[T]` or `&str` is its own
relocatable object that the linker has to deal with, so compile times, link times and binary size are absurdly high.
I initially optimized this by putting the all strings into one giant string, somewhat like this:

```rust
//This actually comes from an include_str!().
static ALL_TEXT: &str = "ＲＳ２３２ケーブルアールエスにさんにケーブルrs232 cable...";

static ENTRIES: &[EntryRepr] = &[
  ...,
  Entry {
    sequence_number: 1000150,
    kanji_elements: &[
      KanjiElementRepr {
        text: StringRef { start: 0, end: 27 },
      },
    ],
    reading_elements: &[
      ReadingElementRepr {
        text: StringRef { start: 27, end: 66 },
      },
    ],
    senses: &[
      SenseRepr {
        parts_of_speech: &[
          PartOfSpeech::Noun,
        ],
        glosses: &[
          GlossRepr {
            text: StringRef { start: 66, end: 77 },
          },
        ],
      },
    ],
  },
  ...
];
```

This helps with the `&str` objects, but there is still the various cascaded `&[T]`. I applied the same technique to
those as well:

```rust
static ALL_TEXT: &str = "ＲＳ２３２ケーブルアールエスにさんにケーブルrs232 cable...";

static ALL_K_ELE: &[KanjiElementRepr] = &[
  KanjiElementRepr {
    text: StringRef { start: 0, end: 27 },
  },
  ...
];

static ALL_R_ELE: &[ReadingElementRepr] = &[
  ReadingElementRepr {
    text: StringRef { start: 27, end: 66 },
  },
  ...
];

static ALL_POS: &[PartOfSpeech] = &[
  PartOfSpeech::Noun,
  ...
];

static ALL_GLOSSES: &[GlossRepr] = &[
  GlossRepr {
    text: StringRef { start: 66, end: 77 },
  },
  ...
];

static ALL_SENSES: &[SenseRepr] = &[
  SenseRepr {
    parts_of_speech: ArrayRef { start: 0, end: 1 },
    glosses: ArrayRef { start: 0, end: 1 },
  },
  ...
];

static ALL_ENTRIES: &[EntryRepr] = &[
  EntryRepr {
    kanji_elements: ArrayRef { start: 0, end: 1 },
    reading_elements: ArrayRef { start: 0, end: 1 },
    senses: ArrayRef { start: 0, end: 1 },
  },
  ...
];
```

With this and the previous sample, you can see that it's not `Entry` anymore, but `EntryRepr` instead, since those
`StringRef` and `ArrayRef` instances need to be resolved into the things they point to at the API boundary. That's why
the actual exposed types use iterators instead of slice refs for everything: to provide a point where this mapping can
take place.

The structure as described above produces binaries of reasonable size, but because all that generated code needs to be
parsed by the compiler, compile times are still frustratingly slow (on the order of minutes for a full build). And
what's worse, the compiler uses so much working memory that my desktop PC with 16 GiB of RAM went OOM trying to compile
it.

To avoid the need for parsing generated code altogether, I finally replaced all `&[TRepr]` arrays with a single
`static ALL_DATA: &[u32]` that gets imported from a binary file via `include_bytes!()`. u32 was chosen because it is
large enough to index into all relevant structures (both `ALL_TEXT` and `ALL_DATA`). I could have encoded enum variants
as u16, but for now, I prefered the simplicity of having everything in one place and accepted the slight inefficiency in
encoding.

Besides `ALL_TEXT` and `ALL_DATA`, there is one final structure, `static ALL_ENTRY_OFFSETS: &[u32]`, which, as an
entrypoint into the self-referencing structure of `ALL_DATA`, provides the offsets into `ALL_DATA` where entries are
located.
