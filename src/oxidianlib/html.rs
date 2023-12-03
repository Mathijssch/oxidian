
//pub enum Tag{
//    Div(Option<String>),
//    A{
//        href: String, 
//        options: Option<String>
//    },
//    Span(String),
//}

pub fn wrap_html_raw(content: &str, tag: &str, options: &str) -> String {
    return format!("<{} {}>{}</{}>", tag, options, content, tag); 
}

pub fn video_tag(src: &str) -> String {
    wrap_html_raw("", "video", &format!("src=\"{}\"", src))
}

pub fn img_tag(src: &str) -> String {
    wrap_html_raw("", "img", &format!("src=\"{}\"", src))
}
//pub fn wrap_html(content: &str, tag: Tag) -> String {
//    match tag {
//        Tag::Div(options) => {format!("<div {}>\n{}\n</div>", options.or(""), content)},
//        Tag::Span(options)
//    }

//}

