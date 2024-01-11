use regex::Regex;

lazy_static! {
static ref OBSIDIAN_LINK_RE: Regex = 
    Regex::new(r"(?P<is_attachment>!?)\[{2}(?P<link>[^\[]*?)\]{2}")
.unwrap();

static ref MD_LINK_RE: Regex = 
    Regex::new(r"(?P<is_attachment>!?)\[(?P<alias>[^\]]*?)\]\((?P<target>[^\)]*?)\)")
.unwrap();
}
use super::link::Link;

pub fn find_markdown_links(content: &str) -> Vec<Link> {
    MD_LINK_RE.captures_iter(content)
        .map(|capture| Link::from_md_link(
                capture.get(0).map_or("", |s| s.as_str()), 
                &capture["target"],
                Some(&capture["alias"]),
                !capture["is_attachment"].is_empty())
        ).collect()
}

pub fn find_obsidian_links(content: &str) -> Vec<Link> {
    OBSIDIAN_LINK_RE.captures_iter(content)
        .map(|capture| Link::from_obsidian_link(&capture["link"], !capture["is_attachment"].is_empty()).unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::oxidianlib::link::{LinkType, Link, FileType};

    use super::find_obsidian_links;

    fn test_multiple_links(content: &str, links: Vec<&str>) {
        let found_links = find_obsidian_links(content);
        assert_eq!(found_links.len(), links.len());  
        
        for (found_link, given_link) in found_links.iter().zip(
                links.iter().map(
                    |link_str| Link::from_obsidian_link(link_str, false).unwrap())) {
            assert_eq!(*found_link, given_link);
        }
    }

    #[test]
    fn test_basic_link() {
        test_multiple_links("some line with a simple [[link]] to another file.", vec!["link"]);
    }

    #[test]
    fn test_basic_link_spaces() {
        test_multiple_links("some line with a simple [[ link ]] to another file.", vec![" link "]);
    }

    #[test]
    fn test_basic_link_subpath() {
        test_multiple_links("some line with a simple [[ link#header ]] to another file.", vec![" link#header "]);
    }

    #[test]
    fn test_basic_link_alias() {
        test_multiple_links("some line with a simple [[ link#header | alias ]] to another file.", vec![" link#header | alias "]);
    }

    #[test]
    fn test_double_link_with_false_positive() {
        test_multiple_links("some line with a simple [[ link#header | alias ]] to another file. Here is another link [[ But this one is fake , and then a [[real_link]]", vec![" link#header | alias ", "real_link"]);
    }
    #[test]
    fn test_basic_link_attachment() {
        let found_links = find_obsidian_links("A line with an ![[attachment.png]]");
        assert_eq!(found_links.len(), 1);
        assert_eq!(found_links[0].link_type(), LinkType::Attachment(FileType::Image));
    }
    // More tests...
}
