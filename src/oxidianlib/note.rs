use super::{obs_comments, obs_admonitions};
use pulldown_cmark::{html, Event, Options, Parser, Tag};

//Check if a sorted collection of delimiters is balanced.
fn is_balanced_sorted(delimiters: &Vec<Delimiter>) -> bool {
    if delimiters.len() % 2 != 0 {
        return false;
    }

    let mut level = 0;
    for d in delimiters {
        match d {
            Delimiter::Begin(_) => {
                level += 1;
            }
            Delimiter::End(_) => {
                level -= 1;
            }
        }
        if level < 0 {
            return false;
        }
    }
    level == 0
}

fn strip_comments(note: &str) -> String {
    let mut output = String::with_capacity(note.len());
    for line in note.lines() {
        output.push_str(obs_comments::process_line(line));
        output.push('\n');
    }
    return output;
}

fn format_admonitions(note: &str) -> String { 
    let mut admonitions = obs_admonitions::AdmonitionParser::new(); 
    let mut output = String::with_capacity(note.len());
    for line in note.lines() {
        if let Some(new) = admonitions.process_line(line) {
            output.push_str(new.as_str());
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }
    return output;

}


pub fn parse_note(note: &str, opts: Options) -> String {
    let stripped = format_admonitions( 
        &strip_comments(note)
    ); 

    let parser = pulldown_cmark::Parser::new(&stripped);
    let mut html_output = String::new();
    //html::push_html(
        //&mut html_output,
        //events.iter().map(|e| e.clone().to_owned()),
    //);
    html::push_html(&mut html_output, parser);
    return html_output;
}
    


    //let mut events: Vec<&Event>;
    //if let Some(size) = parser.size_hint().1 {
    //    events = Vec::with_capacity(size);
    //} else {
    //    events = vec![];
    //}

    //let mut admonitions = obs_admonitions::ObsAdmonition::new();

    //for event in parser {
    //    admonitions.handle_event(event).for_each(|e| events.push(e));

        //if let Some(processed) = admonitions.handle_event(event)
        //    .and_then(|proc_events| events.iter()
        //              .map(|e| unity.handle_event(e))
        //              .flat_map(|optional_events| optional_events.ok_or(vec!()))
        //              .collect()
        //              )
        //{
        //    events.append(processed);
        //}
    //}
    //events.iter().for_each(print_text_event);
    //parser.for_each(|e| print_text_event(&e));
//    let mut html_output = String::new();
//    ////html::push_html(
//    ////    &mut html_output,
//    ////    events.iter().map(|e| e.clone().to_owned()),
//    ////);
//    //html::push_html(&mut html_output, parser);
//    return html_output;
//}

pub fn print_text_event(e: &Event) {
    println!("{:?}", e);
    //match e {
    //    Event::Text(t) => println!("{}", t),
    //    _ => {}
    //}
}

//impl LaTeXEnv {

//    fn new<T: Into<String>>(env_name: T) -> Self {

//        Self {
//            env_name: env_name.into(),
//            content: String::new()
//        }
//    }

//    fn start_pattern(&self) -> String {
//        return format!("\\begin{{{}}}", self.env_name);
//    }

//    fn end_pattern(&self) -> String {
//        return format!("\\end{{{}}}", self.env_name);
//    }

//}

#[derive(PartialEq, Eq, Debug)]
enum Delimiter {
    Begin(usize),
    End(usize),
}

impl PartialOrd for Delimiter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Delimiter::Begin(value1), Delimiter::Begin(value2))
            | (Delimiter::Begin(value1), Delimiter::End(value2))
            | (Delimiter::End(value1), Delimiter::Begin(value2))
            | (Delimiter::End(value1), Delimiter::End(value2)) => value1.partial_cmp(value2),
        }
    }
}

impl Ord for Delimiter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Delimiter::Begin(value1), Delimiter::Begin(value2))
            | (Delimiter::Begin(value1), Delimiter::End(value2))
            | (Delimiter::End(value1), Delimiter::Begin(value2))
            | (Delimiter::End(value1), Delimiter::End(value2)) => value1.cmp(value2),
        }
    }
}
