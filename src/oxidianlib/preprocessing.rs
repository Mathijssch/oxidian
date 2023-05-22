use pulldown_cmark::{Parser, Options, html, Event, CowStr};
use std::iter::Once;

///Trait for preprocessing a markdown file. Allows custom handling of a 
///[pulldown_cmark] [Event]. The default is the identity mapping, which 
///simply passes through the given event.
pub trait Preprocess {
    type IteratorType<'a>: Iterator<Item=Event<'a>>;

    fn process_parser<'a, I>(events: I) -> Self::IteratorType<'a> 
        where I: Iterator<Item=Event<'a>>;

}

///Dummy preprocessor which simply prints the given event with the type 
pub struct PrintEvent; 

impl Preprocess for PrintEvent{ 
    type IteratorType<'a> = Once<&'a Event<'a>>;
    fn handle_event<'a> (&mut self, event: Event<'a>) -> Self::IteratorType<'a>
    {
        println!("{:?}", event);
        return std::iter::once(&event);
    } 
}

