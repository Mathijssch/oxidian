# Convert Obsidian-flavored Markdown notes to HTML

## To-do 
- [ ] Detect broken links
- [ ] Properly differentiate between absolute and relative paths
    - The above two could be merged:
    -   First assume the link is relative and check existence of the file. 
    -   If not found, try absolute. 
    -   If still not found, then mark as a broken link.
- [ ] (Link previews)
- [x] Generate tag overview pages 
- [x] Populate navbar
- [ ] Generate timeline page
- [ ] Build search index
- [ ] [performance]: cache the backlinks.
    - [ ] Loop over the recently modified notes, and for each, loop over the keys, and just add/remove accordingly.



## Disclaimer 

- This is very much a work in progress and is neither efficient, nor complete. 
- There are still many `unwrap()` statements in the code, which should be replaced by proper error handling.

## Simple approach 

For all files in the directory 
- If the file is a markdown file
    - Parse it as a note
- Else
    - copy it to the output (if necessary) 


- Parse link
    - determine the type


## Minimal approach

1. Identify the index.

Routine: 
```
parse(note){ 
    identify links 
    for each link
        parse link
}
```
2. Parse the index. 
    - Identify the links.
    - For each link, repeat 
