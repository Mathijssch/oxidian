use regex::Regex;
use log::debug;

lazy_static! {
    static ref OBSIDIAN_LABEL_RE: Regex =
        Regex::new(r"^[\s]*(?P<start>\^)(?P<label>[aA-zZ\d-]*)\n").unwrap();
            //beginning of line, or space
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockLabel {
    pub label: String, 
    pub source: String
}

///Find labels for blockrefs: `^...`
pub fn find_labels(content: &str) -> Vec<BlockLabel> {
    OBSIDIAN_LABEL_RE.captures_iter(content)
        .map(|capture| BlockLabel { 
            label: String::from(&capture["label"]),
            source: format!("{}{}", &capture["start"], &capture["label"])
        })
        .map(|l| {debug!("Found label {}", l.label); l})
        .collect::<Vec<BlockLabel>>()
}

