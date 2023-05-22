# Convert Obsidian-flavored Markdown notes to HTML

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
