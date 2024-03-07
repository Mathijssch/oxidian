/// Obsidian headers do not necessarily have an empty line in front. 
/// In order to have these headers be detected by a markdown parser, 
/// we have to add this leading empty line whenever it's missing.
use regex::Regex;



pub struct HeaderParser {
    state: HeaderState,
}

enum HeaderState {
    Idle,
    PrecedingEmptyLine,
    PrecedingRegLine,
}

lazy_static! {
    static ref OBS_HEADER_RE: Regex = Regex::new(r"^[[:blank:]]?#+[[:blank:]]*").unwrap();
    static ref EMPTYLINE: Regex = Regex::new(r"^[[:blank:]]*$").unwrap();
}

impl HeaderParser {
    pub fn new() -> Self {
        HeaderParser {
            state: HeaderState::Idle,
        }
    }

    pub fn process_line(&mut self, line: &str) -> Option<String> {
        if EMPTYLINE.is_match(line){
            self.state = HeaderState::PrecedingEmptyLine;
            return None;
        }

        // The line is not an empty line. Check if it's a header. 
        if !OBS_HEADER_RE.is_match(line) {
            // The line is not a header and not an empty line. 
            // Thus, it is a regular line.
            // We just take note and make no changes.
            self.state = HeaderState::PrecedingRegLine;
            return None;
        }

        // We are at a header. 
        // This is equivalent to being in the beginning of a file, so set back to idle.
        // Add an empty line if necessary.
        //debug!("Found header {}", line);
        match self.state {
            HeaderState::PrecedingRegLine => {
                self.state = HeaderState::Idle;
                // If the preceding line is just a regular line, 
                // add an empty line.
                Some(format!("\n{}", line))
            }, 
            _ => {
                self.state = HeaderState::Idle;
                None
            }
        }
    }
}

