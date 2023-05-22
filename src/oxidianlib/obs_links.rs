use regex::Regex;

lazy_static! {
static ref OBSIDIAN_LINK_RE: Regex = 
    Regex::new(r"(?P<is_attachment>!?)\[{2}(?P<link>.*?)\]{2}")
.unwrap();
}
use super::link::Link;


pub fn find_obsidian_links(content: &str) -> Vec<Link> {
    OBSIDIAN_LINK_RE.captures_iter(content)
        .map(|capture| Link::from_obsidian_link(&capture["link"], !capture["is_attachment"].is_empty()).unwrap())
        .collect()
}
