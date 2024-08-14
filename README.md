<div align="center">
    <h1>Oxidian</h1>
    <i>Obsidian-style note-taking from any text editor</i>
</div>
<hr>

Write [Obsidian](https://obsidian.md/)-flavored Markdown in any text editor (including Obsidian itself)
and preview an html preview in real-time.

## Some additional features 

- Automatically build an archive page with all the notes in the Vault sorted chronologically.
- Automatically build an index page for all the tags that appear throughout the notes.

## Installation

### Build from source using Cargo

Clone the repository and simply use `cargo build` or `cargo install --path .` 

### Download prebuilt binaries

TODO

## Usage 

Once `oxidian` is installed, you can run it using
```
oxidian build <notes_directory>
```

Check `oxidian --help` to get more information about the available commands 
and their arguments.

## Configuration 

### Config file

Several settings can be set in a `config.toml` file, which `oxidian` looks for 
in the root of your notebook.

TODO: document the configuration

### Logging

Setting the `RUST_LOG` environment variable controls the logging level.
For instance, to get some feedback on what `oxidian` is doing, 
run `export RUST_LOG=info` before calling `oxidian`.



## Alternatives 
There are many options. For instance, check out [Obsidian awesome](https://github.com/kmaasrud/awesome-obsidian?tab=readme-ov-file#publishing).

## To-do 
- [x] Detect broken links
- [x] Properly differentiate between absolute and relative paths
    - The above two could be merged:
    -   First assume the link is relative and check existence of the file. 
    -   If not found, try absolute. 
    -   If still not found, then mark as a broken link.
- [ ] (Link previews)
- [x] Handle size arguments in included figures
- [x] Generate tag overview pages 
- [x] Populate navbar
- [ ] Refactor so that the exporter only handles io. This makes it easier to unit test operations independently from the file system.
- [~] Unify the way the filenames for tags are generated. See [filenames for tags](#tags).
- [ ] Use Handlebars for templating
- [x] Generate timeline page
- [x] Build search index: see `search.rs`. 
- [ ] [performance]: cache the backlinks.
    - [ ] Loop over the recently modified notes, and for each, loop over the keys, and just add/remove accordingly.
- [ ] [performance]: Replace as many `String`s as possible with `Cow<Str>`s.
- [ ] [performance]: Search the location of a file by first copying the file tree to memory. This will save many syscalls.
- [ ] [performance]: Use [AhoCorasick](https://docs.rs/aho-corasick/latest/aho_corasick/struct.AhoCorasick.html) crate for multiple replacements in a string.

## Warning

- This is very much a work in progress and is neither efficient, nor complete. 
- There are still many `unwrap()` statements in the code, which should be replaced by proper error handling.
