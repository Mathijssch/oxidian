use std::path::Path;

use super::link::Link;
use super::{obs_admonitions, obs_comments, obs_links};
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use super::utils::read_note_from_file;


pub struct Note<'a> {
    pub path: &'a Path, 
    pub links: Vec<Link>,
    content: Option<&'a str>,
    title: Option<String>,  
}


impl<'a> Note<'a> { 
   
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, std::io::Error> {
        return Self::new_inner(path.as_ref());
    }


    fn new_inner(path: &'a Path) -> Result<Self, std::io::Error> {
        let content = Self::sanitize(&read_note_from_file(path)?);  
        let links = Self::find_obsidian_links(&content);
        

    }
        
    fn find_obsidian_links(content: &str) -> Vec<Link> {
        obs_links::find_obsidian_links(content);
          

    }

    fn sanitize(content: &str) -> String { 
        return format_admonitions(&strip_comments(content));
    }


}







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
    let stripped = format_obsidian_links(
        &format_admonitions(
            &strip_comments(note)
            )
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
