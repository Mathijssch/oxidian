use crate::core::html::wrap_html_raw;
use crate::core::sanitization::Sanitization;

pub struct RawHTMLParser {
    state: RawHTMLParseState,
}

pub enum ParseOutput {
    Placeholder {
        replacement: String,
        placeholder: Option<Sanitization>,
    },
    None,
}

enum RawHTMLParseState {
    Idle,
    Body,
}

impl RawHTMLParser {
    pub fn new() -> Self {
        RawHTMLParser {
            state: RawHTMLParseState::Idle,
        }
    }

    pub fn process_line(&mut self, line: &str) -> ParseOutput {
        let trimmed_line = line.trim_start();
        match self.state {
            RawHTMLParseState::Idle => {
                if trimmed_line == "%%RAW_HTML" {
                    self.state = RawHTMLParseState::Body;
                    let replacement = "".to_string();
                    let sanitization = Sanitization::new(line, replacement, false);
                    ParseOutput::Placeholder {
                        replacement: sanitization.get_placeholder(),
                        placeholder: Some(sanitization),
                    }
                } else {
                    ParseOutput::None
                }
            }
            RawHTMLParseState::Body => {
                if trimmed_line == "%%RAW_HTML" {
                    self.state = RawHTMLParseState::Idle;
                    let replacement = "".to_string();
                    let sanitization = Sanitization::new(line, replacement, false);
                    return ParseOutput::Placeholder {
                        replacement: sanitization.get_placeholder(),
                        placeholder: Some(sanitization),
                    };
                }
                let sanitization = Sanitization::new(line, line, false);
                let replacement = sanitization.get_placeholder();
                let placeholder = Some(sanitization);
                return ParseOutput::Placeholder {
                    replacement,
                    placeholder,
                };
            }
        }
    }
}
