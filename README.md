# Oxidian 

Translate Obsidian-style notes to html.


## Features 

- Automatically build an archive page with all the notes in the Vault sorted chronologically.
- Automatically build an index page for all the tags that appears throughout the notes.


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

However, this tool was written to allow me to disconnect from the limitations 
of Obsidian, and allow me to implement my own custom extensions.  


## To-do 
- [x] Detect broken links
- [x] Properly differentiate between absolute and relative paths
    - The above two could be merged:
    -   First assume the link is relative and check existence of the file. 
    -   If not found, try absolute. 
    -   If still not found, then mark as a broken link.
- [x] Have an option to find a file, even if the full path is not specified correctly. 
    - If the path consists of only one component, i.e., `filename`, do a recursive search over the input directory for `filename`. If it can be found, replace the path by this path. Otherwise, mark it as broken. 
    - This is best to be optional though, since it will make compilation slower.  
    ~~It remains to test how big the impact of this is.~~ It is quite significant.
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
- [x] Automatically refer to the tag index page, when a tag is detected.
- [x] Test if checklists are correctly handled.
- [x] Implement decent styling and frontend functionality in default template.
    - [x] Mathjax / KaTeX support 
        - [x] Convert .sty preamble to mathjax config
    - [x] Navbar styling
    - [x] Avoid FOUC
    - [x] Add the counts to the tags in the tree


## Warning

- This is very much a work in progress and is neither efficient, nor complete. 
- There are still many `unwrap()` statements in the code, which should be replaced by proper error handling.
