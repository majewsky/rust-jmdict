# WARNING: Licensing on database files

The database files compiled into the crate are licensed from the Electronic Dictionary Research and Development Group
under Creative Commons licenses. Applications linking this crate directly oder indirectly must display appropriate
copyright notices. Please refer to the [EDRDG's license statement](https://www.edrdg.org/edrdg/licence.html) for details.

# rust-jmdict

![GitHub Actions Badge](https://github.com/majewsky/rust-jmdict/actions/workflows/test.yml/badge.svg)

The `jmdict` crate contains the data from the [JMDict file](https://www.edrdg.org/jmdict/j_jmdict.html), a comprehensive
multilingual dictionary of the Japanese language. The original JMDict file, included in this repository (and hence, in
releases of this crate) comes as XML. Instead of stuffing the XML in the binary directly, this crate parses the XML at
compile-time and generates an optimized representation for inclusion in the final binary.

In short, this crate does:

* parse the XML structure of the JMdict database file,
* provide an API to access its entries, and
* provide compile-time flags (via Cargo features) to select the amount of information included in the binary.

This crate does NOT:

* provide fast lookup into the database. You get a list of entries and then you can build your own indexing on top as
  required by your application.

For specific examples, please check out the [documentation on docs.rs](https://docs.rs/jmdict/).

## Building

When packaging to crates.io, we cannot include the actual payload data (`data/entrypack.json`) because crates.io has a
limit of 10 MiB per crate. (Technically, we could ship the data by depending on a series of data crates each slightly
under 10 MiB, but I intend to be a good citizen and not abuse the shared infrastructure of crates.io needlessly.)

Hence the default strategy is to pull the entrypack (a preprocessed form of the JMdict contents) at build time from
a server under the crate owner's control, currently <https://dl.xyrillian.de/jmdict/>. Each released crate version will
have the most recent entrypack (as of the time of publication) hardcoded into its code, along with a SHA-256 checksum to
ensure data integrity.

If downloading the entrypack at build time is not possible (e.g. because the build machine does not have internet
access, or because `curl` is not installed on the build machine), download the entrypack beforehand and put its path in
the `RUST_JMDICT_ENTRYPACK` environment variable when running `cargo build`.

For development purposes, when building from the repository, `data/entrypack.json` will be used instead. If this is not
desired, set the value of the `RUST_JMDICT_ENTRYPACK` to `default` to force the normal download behavior.

## Contributing

If you plan to open issues or write code, please have a look at [CONTRIBUTING.md](CONTRIBUTING.md).
