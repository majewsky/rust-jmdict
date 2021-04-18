# jmdict-traverse

Parsing utilities for the build and test phases of the `jmdict` crate.

This code is in a separate crate because, if we put it in the `jmdict` crate itself, its `build.rs`
could not import it.

## Compatibility promise

**There is none.** Although this crate is published on crates.io for technical reasons, this crate
is internal to the `jmdict` crate. Its API may change at any time, including in bugfix releases. Use
the [API provided by the `jmdict` crate](https://docs.rs/jmdict/) instead.
