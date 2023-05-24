use std::path::Path;

use super::frontmatter::{extract_yaml_frontmatter, parse_frontmatter};
use super::link::Link;
use super::{obs_admonitions, obs_comments, obs_links, frontmatter};
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use serde::__private::de::Content;
use yaml_rust::Yaml;
use super::utils::read_note_from_file;


#[derive(Debug)]
pub struct Note<'a> {
    pub path: &'a Path, 
    pub links: Vec<Link>,
    pub frontmatter: Option<Yaml>,
    content: Option<&'a str>,
    title: String,  
}

impl<'a> Note<'a> { 

    fn get_author_prefix(frontmatter: &Yaml) -> Option<String> {
        if let Some(author) = frontmatter["authors"][0].as_str() {
            if let Some(year) = frontmatter["year"].as_str() {
                Some(format!("{} ({}) -", author, year))
            } else {
                Some(format!("{} -", author))
            }; 
        };
        None
    }


    fn get_title(filename: &Path, frontmatter: Option<&Yaml>) -> String {
        let base_title = match frontmatter
            .and_then(
                 |fm| fm["title"].as_str()
            ) 
            {
                Some(title) => title, 
                None => 
                {
                    filename.file_stem().and_then(|f| f.to_str()).unwrap_or("Note")
                }
            };
        let prefix = frontmatter
            .and_then(|fm| Self::get_author_prefix(fm))
            .unwrap_or_else(|| String::from(""));
        return prefix + base_title;
    }

    pub fn new(path: &'a Path) -> Result<Self, std::io::Error> {
        let content = Self::sanitize(&read_note_from_file(path)?);
        let frontmatter = extract_yaml_frontmatter(&content)
            .and_then(|fm| parse_frontmatter(&fm).ok());
        let links = Self::find_obsidian_links(&content);
        let title = Self::get_title(path, frontmatter.as_ref());
        Ok(Note{
            path, links, content: None, title, frontmatter
        })
    }
        
    fn find_obsidian_links(content: &str) -> Vec<Link> {
        obs_links::find_obsidian_links(content) 
    }

    fn sanitize(content: &str) -> String { 
        return format_admonitions(&strip_comments(content));
    }


}







//Check if a sorted collection of delimiters is balanced.
//fn is_balanced_sorted(delimiters: &Vec<Delimiter>) -> bool {
//    if delimiters.len() % 2 != 0 {
//        return false;
//    }

//    let mut level = 0;
//    for d in delimiters {
//        match d {
//            Delimiter::Begin(_) => {
//                level += 1;
//            }
//            Delimiter::End(_) => {
//                level -= 1;
//            }
//        }
//        if level < 0 {
//            return false;
//        }
//    }
//    level == 0
//}

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


pub fn create_note(path: &str) -> Note {
    let the_path = Path::new(path);
    Note::new(the_path).unwrap()
}

//pub fn parse_note(note: &str, opts: Options) -> String {
//    //let stripped = format_obsidian_links(
//        //&format_admonitions(
//            //&strip_comments(note)
//            //)
//        //); 
//    let parser = pulldown_cmark::Parser::new(&stripped);
//    let mut html_output = String::new();
//    //html::push_html(
//    //&mut html_output,
//    //events.iter().map(|e| e.clone().to_owned()),
//    //);
//    html::push_html(&mut html_output, parser);
//    return html_output;
//}

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
