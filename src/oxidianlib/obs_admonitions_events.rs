//use super::preprocessing::Preprocess;
use pulldown_cmark::{Event,Tag,CowStr, InlineStr};

pub struct ObsAdmonition<'a> {
    state: AdmonitionState,
    description: Option<String>,
    buffer: Vec<Event<'a>>,
    level: u8
}

#[derive(Debug)]
pub enum AdmonitionState { 
    Idle, 
    ExpectOpenBracket,
    ExpectCloseBracket,
    ExpectDescription,
    Open, 
    Resetting
}

impl<'a> ObsAdmonition<'a> {
    pub fn new() -> Self {
        return ObsAdmonition { state: AdmonitionState::Idle,
        buffer: Vec::with_capacity(5), 
        description: None,
        level: 0
    };
    }

    fn transition(&mut self, state: AdmonitionState){
        self.state = state;
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.state = AdmonitionState::Idle; 
    }
}

impl<'a> ObsAdmonition<'a> {
    
    pub fn process<T: Iterator<Item=Event<'a>>>(&mut self, events: T) -> Vec<Event<'a>>{
        let mut result = vec!();
        for event in events { 
            if let Some(processed) = self.handle_event(event){
                result.extend(processed)
            }
        }
        return result; 
    }

    fn handle_event (&mut self, event: Event<'a>) -> Option<Vec<Event<'a>>> {
        println!("Event: {:?} state: {:?}", event, self.state);
        match self.state {
            AdmonitionState::Resetting => {self.reset()},
            _ => ()
        }
        self.buffer.push(event.clone());
        match self.state {
            AdmonitionState::Idle => {
                match event {
                    Event::Start(Tag::BlockQuote) => {
                        self.level += 1;
                        self.transition(AdmonitionState::ExpectOpenBracket);
                    },
                    _ => {
                        self.buffer.clear();
                        return Some(vec!(event));
                    }
                }
            },
            AdmonitionState::ExpectOpenBracket => {
                match event {
                    Event::Start(Tag::Paragraph) => (),
                    Event::Text(CowStr::Borrowed("[")) => {
                        self.transition(AdmonitionState::ExpectDescription);
                    },
                    _ => {
                        self.transition(AdmonitionState::Resetting);
                        return Some(self.buffer.to_owned()); 
                    }
                }
            }, 
            AdmonitionState::ExpectDescription => {
               match event {
                    Event::Text(CowStr::Borrowed(text_content)) => {
                        if text_content.starts_with("!") 
                            && text_content.len() >1 {
                                let descriptor = &text_content[1..];
                                self.description = Some(String::from(descriptor));
                                self.transition(AdmonitionState::ExpectCloseBracket);
                        } else {
                            self.transition(AdmonitionState::Resetting);
                            return Some(self.buffer.to_owned());  
                        } 
                    },
                _ => ()
               }
            },
            AdmonitionState::ExpectCloseBracket => {
                match event { 
                    Event::Text(CowStr::Borrowed("]")) => { 
                        self.transition(AdmonitionState::Open);
                        self.buffer.clear();
                        //let  = self.description.unwrap().as_ref(); 
                        self.buffer.pop();
                        self.buffer.push(Event::Html(CowStr::Borrowed("<div class=\"admonition-{}\">"))); 
                    }, 
                    _ => ()
                }
            }, 
            AdmonitionState::Open => {
                match event {
                    Event::Start(Tag::BlockQuote) => {
                        self.level += 1;
                    },
                    Event::End(Tag::BlockQuote) => {
                        self.level -= 1; 
                        if self.level == 0 {
                            println!("All done!"); 
                            self.buffer.pop();
                            self.buffer.push(Event::Html(CowStr::Borrowed("</div>")));
                            self.transition(AdmonitionState::Resetting);
                            return Some(self.buffer.to_owned());
                        }
                    },
                    _ => ()
                }
            }, 
            _ => ()
        }
        return None;
    }
}
