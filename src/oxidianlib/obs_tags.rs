use regex::Regex;

lazy_static! {
    static ref OBSIDIAN_TAG_RE: Regex =
        Regex::new(r"(?:^|\s)#(?P<tagname>[a-zA-Z0-9_-]+(?:/[a-zA-Z0-9_-]+)*)").unwrap();
            //beginning of line, or space
}


#[derive(PartialEq, Eq, Debug)]
pub struct Tag {
    pub tag_path: String, 
    pub source: String
}

pub fn find_tags(content: &str) -> Vec<Tag> {
    let mut tags = OBSIDIAN_TAG_RE.captures_iter(content)
        .filter(|capture| capture.get(0)
                            .map_or(false, 
                                |c| content[c.start()..c.end()].chars().any(|c| c.is_alphabetic()))
                            )
        .map(|capture| Tag { 
            tag_path: String::from(&capture["tagname"]).to_lowercase(),
            source: capture.get(0)
                .map_or("", |m| m.as_str().trim_start())
                .to_string(),
        })
        .collect::<Vec<Tag>>();
    // Sort by length (longest first) to fix issues pertaining to tags beginning with the same word.
    sort_by_length(&mut tags);
    tags
}

fn sort_by_length(elements: &mut Vec<Tag>) {
    elements.sort_by(|b, a| a.tag_path.len().cmp(&b.tag_path.len()));
}


#[cfg(test)]
mod tests {

    use super::{find_tags, Tag};

    fn test_multiple_tags(content: &str, tags: Vec<Tag>) {
        let found_tags = find_tags(content);
        assert_eq!(found_tags.len(), tags.len());  
        
        found_tags.iter()
            .zip(tags.iter())
            .for_each(|(found_tag, given_tag)| {assert_eq!(found_tag, given_tag);});
        }

    #[test]
    fn test_basic_tag() {
        test_multiple_tags("some line with a #tag.", vec![Tag{ tag_path: "tag".to_string(), source: "#tag".to_string() }]);
    }

    #[test]
    fn test_basic_tag_with_space() {
        test_multiple_tags("Something with a false # postiive and a #true_positive",
            vec![Tag{ tag_path: "true_positive".to_string(), source: "#true_positive".to_string()} ]);
    }

    #[test]
    fn test_basic_tag_linebreak() {
        test_multiple_tags("A line with a #tag\nthat is at the end of a line",
            vec![Tag{tag_path: "tag".to_string(), source: "#tag".to_string()}]);
    }
    
    #[test]
    fn test_tag_special_characters() {
        // Note the order! The tags are sorted by length!
        test_multiple_tags("A line with a #tag2 and #tag_2 and #tag3#tag4 and #tag\\5",
            vec![Tag{tag_path: "tag_2".to_string(), source: "#tag_2".to_string()},  
                 Tag{tag_path: "tag2".to_string(), source: "#tag2".to_string()},
                 Tag{tag_path: "tag3".to_string(), source: "#tag3".to_string()},
                 Tag{tag_path: "tag".to_string(), source: "#tag".to_string()},
            ]);
    }

    #[test]
    fn invalid_tags() {
        // Note the order! The tags are sorted by length!
        test_multiple_tags("A line with an invalid #2 and valid #tag2 and another #1902 invalid one: #@test or ##not-a-tag",
            vec![Tag{tag_path: "tag2".to_string(), source: "#tag2".to_string()}]);
    }
}
