use super::html::wrap_html_raw;
use regex::Regex;

lazy_static! {
    static ref OBS_ADMONITION_TITLE_RE: Regex =
        Regex::new(r"\s*(?:\[!(?P<type>[aA-zZ)]+)\])(?P<title>[^\n*]*)").unwrap();
}

pub struct AdmonitionParser {
    state: AdmonitionState,
}

enum AdmonitionState {
    Idle,
    Body,
    SingleBreak
}

impl AdmonitionParser {

    pub fn new() -> Self { 
        return AdmonitionParser {state: AdmonitionState::Idle};
    }

    fn start_admonition(ad_type: &str, title: &str) -> String {
        return format!("<div class=\"admonition admonition-note {}\">\n", ad_type)
             + wrap_html_raw(
            title,
            "div",
            format!("class=\"admonition-title {}\"", ad_type).as_str()
            ).as_str();
    }

    fn exit(&mut self, line: &str) -> Option<String> {
        self.state = AdmonitionState::Idle; 
        return Some(line.to_string() + "\n</div>"); 
    }
    pub fn process_line(&mut self, line: &str) -> Option<String> {
        match self.state {
            AdmonitionState::Idle => {
                if let Some(captures) = OBS_ADMONITION_TITLE_RE.captures(line) {
                    let ad_type = captures
                        .name("type")
                        .map(|v| v.as_str())
                        .expect("Wrong regex!");
                    let title = captures
                        .name("title")
                        .map(|v| v.as_str())
                        .expect("Wrong regex!");
                    self.state = AdmonitionState::Body;
                    let result = Self::start_admonition(ad_type, title);
                    return Some(result);
                } else {
                    return None;
                }
            },
            AdmonitionState::Body | AdmonitionState::SingleBreak => {
                let trimmed_line = line.trim_start();
                if trimmed_line.starts_with(">") {
                    return Some(trimmed_line[1..].to_string());
                } else 
                {
                    match self.state {
                        AdmonitionState::Body => {
                            self.state = AdmonitionState::SingleBreak;
                            return None;
                        }, 
                        AdmonitionState::SingleBreak => {
                            return self.exit(line);
                        },
                        _ => {panic!("Impossible.");}
                    }
                }

            }
        }
    }
}
