use regex::Regex;

lazy_static! {
    static ref OBSIDIAN_TAG_RE: Regex =
        Regex::new(r"#(?P<tagname>[a-zA-Z0-9_-]+(?:/[a-zA-Z0-9_-]+)*)").unwrap();
}

pub fn find_tags(content: &str) -> Vec<String> {
    let mut tags = OBSIDIAN_TAG_RE.captures_iter(content)
        .map(|capture| String::from(&capture["tagname"]).to_lowercase())
        .collect::<Vec<String>>();
    // Sort by length (longest first) to fix issues pertaining to tags beginning with the same word.
    sort_by_length(&mut tags);
    tags
}

fn sort_by_length(elements: &mut Vec<String>) {
    elements.sort_by(|b, a| a.len().cmp(&b.len()));
}


#[cfg(test)]
mod tests {

    use super::find_tags;

    fn test_multiple_tags(content: &str, tags: Vec<String>) {
        let found_tags = find_tags(content);
        assert_eq!(found_tags.len(), tags.len());  
        
        found_tags.iter()
            .zip(tags.iter())
            .for_each(|(found_tag, given_tag)| {assert_eq!(found_tag, given_tag);});
        }

    #[test]
    fn test_basic_tag() {
        test_multiple_tags("some line with a #tag.", vec!["tag".to_string()]);
    }

    #[test]
    fn test_basic_tag_with_space() {
        test_multiple_tags("Something with a false # postiive and a #true_positive", vec!["true_positive".to_string()]);
    }

    #[test]
    fn test_basic_tag_linebreak() {
        test_multiple_tags("A line with a #tag\nthat is at the end of a line", vec!["tag".to_string()]);
    }
    
    #[test]
    fn test_tag_special_characters() {
        // Note the order! The tags are sorted by length!
        test_multiple_tags("A line with a #tag2 and #tag_2 and #tag3#tag4 and #tag\\5", vec![
        "tag_2".to_string(), "tag2".to_string(), "tag3".to_string(), "tag4".to_string(), "tag".to_string() ]);
    }
}
