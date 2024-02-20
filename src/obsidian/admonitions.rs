use crate::core::html::wrap_html_raw;
use crate::core::sanitization::Sanitization;
use regex::Regex;

lazy_static! {
    static ref OBS_ADMONITION_TITLE_RE: Regex =
        Regex::new(r"\s*(?:\[!(?P<type>[aA-zZ)]+)\])(?P<title>[^\n*]*)").unwrap();
    static ref OBS_HEADER_RE: Regex = Regex::new(r"^[[:blank:]]?#+\s").unwrap();
}

pub struct AdmonitionParser {
    state: AdmonitionState,
}


pub enum ParseOutput {
    Placeholder { 
        replacement: String, 
        placeholder: Option<Sanitization>
    },
    None,
}

enum AdmonitionState {
    Idle,
    Body,
    SingleBreak,
}

impl AdmonitionParser {
    pub fn new() -> Self {
        AdmonitionParser {
            state: AdmonitionState::Idle,
        }
    }

    fn start_admonition(ad_type: &str, title: &str) -> String {
        let mut title_html = format!("<div class=\"admonition admonition-note {}\">\n", ad_type);
        title_html.push_str(
            wrap_html_raw(
                title,
                "div",
                format!("class=\"admonition-title {}\"", ad_type),
            )
            .as_ref(),
        );
        title_html.push_str("<div class=\"admonition-content\">");
        title_html
    }


    pub fn process_line(&mut self, line: &str) -> ParseOutput {
        match self.state {
            AdmonitionState::Idle => {
                if let Some(captures) = OBS_ADMONITION_TITLE_RE.captures(line) {
                    let ad_type = captures
                        .name("type")
                        .map(|v| v.as_str())
                        .expect("Wrong regex!")
                        .to_lowercase();
                    let title = captures
                        .name("title")
                        .map(|v| v.as_str())
                        .expect("Wrong regex!");
                    self.state = AdmonitionState::Body;
                    let replacement = Self::start_admonition(&ad_type, title);
                    let sanitization = Sanitization::new(line, replacement, false); 
                    ParseOutput::Placeholder { 
                        replacement: sanitization.get_placeholder(), 
                        placeholder: Some(sanitization)
                    }
                } else {
                    ParseOutput::None
                }
            },
            AdmonitionState::Body | AdmonitionState::SingleBreak => {
                let trimmed_line = line.trim_start();
                if trimmed_line.starts_with(">") {
                    ParseOutput::Placeholder { 
                        replacement: trimmed_line[1..].to_string(),
                        placeholder: None 
                    }
                } else {
                    match self.state {
                        AdmonitionState::Body => {
                            if OBS_HEADER_RE.is_match(line) {
                                // Detected a header. This is a special case,
                                // in which the admonition is ended and the header is
                                // immediately placed.
                                //
                                self.state = AdmonitionState::Idle;
                                // Add a closing div tag, but sanitize it away, so it doesn't 
                                // confuse the markdown renderer. 
                                // That is, `</div>` will be replaced with a hash.
                                // Since the flag `false` is passed, 
                                // it will be substituted back AFTER markdown compilation.
                                // The original line simply gets added back to the content
                                // unchanged.
                                let addition = "</div></div>";
                                let sanitization = Sanitization::new(
                                        addition.to_string(), 
                                        addition.to_string(), false);
                                let replacement = sanitization.get_placeholder() + "\n" + line;
                                let placeholder = Some(sanitization);

                                return ParseOutput::Placeholder{
                                        replacement, placeholder 
                                    };
                            }
                            self.state = AdmonitionState::SingleBreak;
                            ParseOutput::None
                        }
                        AdmonitionState::SingleBreak => {
                            // There already was a break before this
                            // line, so a new line that does not
                            // start with `>` automatically breaks
                            // the admonition.
                            self.state = AdmonitionState::Idle; 

                            let addition = "</div></div>";
                            let sanitization = Sanitization::new(addition, addition, false);
                            //let replacement = line.to_string() + "\n" + &sanitization.get_placeholder();
                            let replacement = sanitization.get_placeholder() + "\n" + line;
                            let placeholder = Some(sanitization);

                            ParseOutput::Placeholder{
                                    replacement, placeholder 
                            }
                        }
                        _ => {
                            panic!("Impossible.");
                        }
                    }
                }
            }
        }
    }
}
