use regex::Regex;

lazy_static! {
    static ref OBSIDIAN_TAG_RE: Regex =
        Regex::new(r"(?<!\\)#(?P<tagname>[a-zA-Z-_/]+)(?![^[[]*]])").unwrap();
}

pub fn find_tags(content: &str) -> Vec<String> {
    let mut tags = OBSIDIAN_TAG_RE.captures_iter(content)
        .map(|capture| String::from(&capture["tagname"]))
        .collect::<Vec<String>>();
    // Sort by length (longest first) to fix issues pertaining to tags beginning with the same word.
    tags.sort_by(|b, a| a.len().cmp(&b.len()));
    tags
}

