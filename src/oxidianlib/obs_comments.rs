use super::constants::OBS_COMMENTS;
//use super::preprocessing::Preprocess;
//use pulldown_cmark::{CowStr, Event};

/// Gobble the given keyword.
//pub struct ObsidianComments {
//    state: CommentState,
//}

//enum CommentState {
//    Idle,
//    Active,
//}

//impl ObsidianComments {
//    pub fn new() -> Self {
//        ObsidianComments {
//            state: CommentState::Idle,
//        }
//    }
//}


pub fn process_line(line: &str) -> &str {
    if let Some(comment_pos) = line.find(OBS_COMMENTS) { 
        return &line[..comment_pos]; 
    }
    return line;
}


//impl Preprocess for ObsidianComments {
//    fn handle_event<'a>(&mut self, event: Event<'a>) -> Option<Event<'a>> {
//        match self.state {
//            CommentState::Idle => {
//                match event {
//                    Event::Text(CowStr::Borrowed(text_content)) => { 
//                        if let Some(comment_pos) = text_content.find(OBS_COMMENTS) 
//                        { // Found a comment string.
//                            println!("gobble");
//                            self.state = CommentState::Active;
//                            return Some(
//                                Event::Text(
//                                    CowStr::Borrowed(&text_content[..comment_pos])
//                                    )
//                                );
//                        } else {
//                            return Some(event);
//                        }
//                    },
//                    _ => {return Some(event);}
//                }
//            }, 
//            CommentState::Active => { 
//                println!("The comment state is Active.");
//                match event {
//                    Event::SoftBreak | Event::HardBreak => {
//                        self.state = CommentState::Idle;
//                        println!("Removing {:?}", event);
//                        return None;
//                    },
//                    _ => {
//                        println!("Removing {:?}", event);
//                        return None;}
//                }
//            }
//        };
//    }
//}
