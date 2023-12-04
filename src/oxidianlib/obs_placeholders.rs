//use super::errors::MathFindError;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher}, cmp,
};

#[derive(Hash,Debug)]
pub struct Sanitization(pub String);

impl Sanitization {
    pub fn get_placeholder(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return format!("{}", hasher.finish());
    }
}

enum MathState {
    ExpectOpen,
    ExpectClose,
}

struct DelimPair {
    pub open: String,
    pub close: String,
}

impl DelimPair {
    pub fn new(open: &str, close: &str) -> Self {
        DelimPair {
            open: String::from(open),
            close: String::from(close),
        }
    }
}

fn get_search_pattern<'a>(state: &MathState, pair: &'a DelimPair) -> &'a str {
    match state {
        MathState::ExpectOpen => &pair.open,
        MathState::ExpectClose => &pair.close,
    }
}

fn find_pairs(content: &str, delim: &DelimPair) -> Vec<Sanitization> {
    let mut result = vec![];
    let mut state = MathState::ExpectOpen;
    let mut curr_start = 0;
    let mut prev_open = None;

    while let Some(index) = content[curr_start..].find(get_search_pattern(&state, &delim)) { 
        let pattern = get_search_pattern(&state, delim);
        match state {
            MathState::ExpectOpen => {
                // Found an opening brace
                state = MathState::ExpectClose;
                prev_open = Some(index);
            },
            MathState::ExpectClose => {
                // Found a closing brace
                if let Some(start_idx) = prev_open {
                    // Retrieve previous opening index.
                    let end_idx = cmp::min(
                        curr_start + index + get_search_pattern(&state, &delim).len(),
                        content.len()-1
                    );
                    result.push(Sanitization(content[start_idx..end_idx].to_owned()));
                    state = MathState::ExpectOpen;
                    prev_open = None;
                }
            }
        };
        curr_start = cmp::min(curr_start + index + pattern.len(), 
                              content.len()-1);
    };
    result
}

pub fn disambiguate_protected(content: &str) -> (String, Vec<Sanitization>) {
    let mut result = vec!();

    let pairs: [DelimPair; 6] = [
        DelimPair::new("$$", "$$"),
        DelimPair::new(r"\[", r"\]"),
        DelimPair::new("$", "$"),
        DelimPair::new(r"\(", r"\)"),
        DelimPair::new(r"```", r"```"),
        DelimPair::new(r"`", r"`")
    ];
    let mut new_string = String::from(content);
    for pair in pairs { 
        let sanitize = find_pairs(&new_string, &pair);
        for math_element in &sanitize { 
            new_string = new_string.replace(
                &math_element.0, &math_element.get_placeholder()
                );
        }
        result.extend(sanitize);
    }
    (new_string, result)
}
