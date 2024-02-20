use slugify::slugify;
use std::marker::PhantomData;
use std::write;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark::escape::StrWrite;
use pulldown_cmark::html::push_html;

pub struct MarkdownParser<'a, P> {
    parser: P,
    _type_hint: PhantomData<&'a P>,
}


impl<'a, P> MarkdownParser<'a, P>
where
    P: Iterator<Item=Event<'a>>,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _type_hint: PhantomData,
        }
    }

    fn convert_heading(&mut self, level: pulldown_cmark::HeadingLevel, classes: Vec<&str>) -> Event<'a> {
        // Read events until the end of heading 
        let mut buffer = Vec::new();
        let mut content_buffer = String::new();

        while let Some(event) = self.parser.next() {
            match &event {
                Event::End(Tag::Heading(n, _, _)) if n == &level => break,
                Event::Text(text) => {
                    write!(content_buffer, "{}", &text)
                    .expect("Could not write text to a string")
                },
                _ => {},
            }
            buffer.push(event.clone()); // Necessary clone, because the inner parser throws them
                                        // out.
        }

        // Convert the events into an HTML Tag
        let mut html = String::with_capacity(content_buffer.capacity());
        write!(&mut html, "<{}", level).unwrap(); 
        let header_id = slugify!(&content_buffer);
        if header_id.len() > 0 {
            write!(&mut html, " id=\"{}\"", header_id).unwrap();
        }
        for class in classes { 
            write!(&mut html, " class=\"{}\"", class).unwrap();
        }
        html.push_str(">");
        push_html(&mut html, buffer.into_iter());
        writeln!(&mut html, "</{}>", level).unwrap();

        Event::Html(html.into())
    }
}

impl<'a, P> Iterator for MarkdownParser<'a, P>
where
    P: Iterator<Item=Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.next() {
            // If we get a header without an id, generate one.
            Some(Event::Start(Tag::Heading(level, None, classes))) => Some(self.convert_heading(level, classes)),
            Some(event) => Some(event),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn convert(s: &str) -> String {
        let mut buf = String::new();
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);
        
        let basic_parser = pulldown_cmark::Parser::new_ext(s, options);
        let parser = MarkdownParser::new(basic_parser);
        pulldown_cmark::html::push_html(&mut buf, parser);
        buf
    }

    #[test]
    fn heading_id() {
        let s = "## Heading {#heading-id}";
        assert_eq!(convert(s).trim_end(), r#"<h2 id="heading-id">Heading</h2>"#);
    }

    #[test]
    fn normal() {
        let s = "## Heading";
        assert_eq!(convert(s).trim_end(), r#"<h2 id="heading">Heading</h2>"#);
    }

    #[test]
    fn inline_code() {
        let s = "# `source code` heading {#source}";
        assert_eq!(convert(s).trim_end(),
            r#"<h1 id="source"><code>source code</code> heading</h1>"#);
    }

    #[test]
    fn em_strong() {
        let s = "## *Italic* __BOLD__ heading {#italic-bold}";
        assert_eq!(convert(s).trim_end(),
            r#"<h2 id="italic-bold"><em>Italic</em> <strong>BOLD</strong> heading</h2>"#);
    }

    #[test]
    fn whitespace() {
        let s = "## ID with space";
        assert_eq!(convert(s).trim_end(),
            r#"<h2 id="id-with-space">ID with space</h2>"#);
    }

    #[test]
    fn empty() {
        assert_eq!(convert("##").trim_end(), "<h2></h2>");
    }
    
    #[test]
    fn with_link() {
        let s = "### [Link](https://example.com/) {#example}";
        assert_eq!(convert(s).trim_end(),
            r#"<h3 id="example"><a href="https://example.com/">Link</a></h3>"#);
    }

    #[test]
    fn to_be_escaped() {
        let s = "## ><";
        assert_eq!(convert(s).trim_end(), "<h2>&gt;&lt;</h2>");
    }
}

