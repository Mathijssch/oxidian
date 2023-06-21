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


#[cfg(test)]
mod tests {
    use crate::oxidianlib::link::Link;

    use super::find_obsidian_links;

    fn test_single_link(content: &str, link: &str) {
        let found_links = find_obsidian_links(content);
        let true_link = Link::from_obsidian_link(link, false).unwrap();
        assert_eq!(found_links.len(), 1);  
        assert_eq!(found_links[0], true_link);
    }


    #[test]
    fn test_basic_link() {
        test_single_link("some line with a simple [[link]] to another file.", "link");
    }

    #[test]
    fn test_basic_link_spaces() {
        test_single_link("some line with a simple [[ link ]] to another file.", " link ");
    }

    #[test]
    fn test_basic_link_subpath() {
        test_single_link("some line with a simple [[ link#header ]] to another file.", " link#header ");
    }

    #[test]
    fn test_basic_link_alias() {
        test_single_link("some line with a simple [[ link#header | alias ]] to another file.", " link#header | alias ");
    }
    // More tests...
}
