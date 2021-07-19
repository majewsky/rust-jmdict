# v2.0.0 (2021-07-19)

JMdict was updated to 2021-09-17. This requires the following changes in enum variants:

- added `Dialect::Brazilian`
- added `GlossType::Trademark`
- added `KanjiInfo::RareKanjiForm`
- added `SenseInfo::Document`
- added `SenseInfo::Group`
- renamed `SenseInfo::{LiteraryOrFormalTerm => FormalOrLiteraryTerm}`
- added `SenseTopic::Clothing`
- added `SenseTopic::Psychiatry`
- added `SenseTopic::Railway`

Further changes:

- The `all_variants()` method was added to `trait Enum`.
- The `from_constant_name()` method was added to `trait Enum`.

# v1.0.0 (2021-04-18)

Initial stable release. No changes from v0.99.1.

# v0.99.1 (2021-04-18)

Initial preview release to test documentation rendering on <https://docs.rs/> and application builds pulling the crate
from <https://crates.io/>.
