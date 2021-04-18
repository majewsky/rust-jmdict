# `data/`

We cannot put the JMdict into Git as one single file because its size is over 100 MiB, and GitHub does not like files
that big. I don't want to use LFS because it's still a text file and thus delta-compresses really well if you let Git do
its job. Therefore we split the file into chunks of roughly 1000 entries each.

Since we're pre-processing anyway, we're also converting from XML to JSON. The original XML file uses a lot of memory
when parsed as a whole, and parsing in pieces is finicky because we need to carry over the DTD into each chunk, if only
for the entity definitions. The JSON files in this directory, on the other hand, do not have any magical entities and
thus trivially parse as individual entries. It also turns out that parsing JSON is much quicker than parsing XML, which
makes a significant impact on the build time of the whole crate.

## Import workflow

To update the JMdict copy in this directory, run `make import JMDICT_PATH=/path/to/JMdict`. Check the `git diff`
afterwards; it should usually only show changes for a few places where upstream edited the respective JMdict entries.

## Export workflow

We cannot bundle the data files with the crates when publishing because crates.io imposes a 10 MiB limit on crates. The
data files are therefore stored in a compressed bundle by `make export`. The output file appears in this directory as
`entrypack-YYYY-MM-DD.json.gz`, with the date being extracted from JMdict's own modification timestamp in
`entries-999.json`.

This file can then be copied to its web server location, currently residing on <http://dl.xyrillian.de/jmdict/> under
the control of [@majewsky](https://github.com/majewsky). Finally, update the constants at the top of
`jmdict-traverse/src/file.rs` to refer to the new file.
