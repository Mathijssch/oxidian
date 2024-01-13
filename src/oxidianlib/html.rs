use std::collections::{BTreeSet, BTreeMap};
use std::path::Path;

/// General utility function to wrap a given string into a html tag.
pub fn wrap_html_raw<T: AsRef<str>, U: AsRef<str>, V: AsRef<str>>(
    content: T,
    tag: U,
    options: V,
) -> String {
    return format!(
        "<{} {}>{}</{}>",
        tag.as_ref(),
        options.as_ref(),
        content.as_ref(),
        tag.as_ref()
    );
}

/// Generate a video tag to the given `src` video.
pub fn video_tag(src: &str) -> String {
    wrap_html_raw("", "video", &format!("src=\"{}\"", src))
}

/// Generate an image tag to the given `src` image.
pub fn img_tag(src: &str) -> String {
    wrap_html_raw("", "img", &format!("src=\"{}\"", src))
}

/// Generate a html link to the given path, with the given `text` as alias.
pub fn link<T: AsRef<Path>>(dst: T, text: &str, options: &str) -> String {
    wrap_html_raw(
        text,
        "a",
        &format!("href=\"{}\" {}", dst.as_ref().to_string_lossy(), options),
    )
}

///Generate an unordered list of elements that can be displayed.
pub fn ul<T: std::fmt::Display, U: Iterator<Item = T>>(src: U, options: &str) -> String {
    wrap_html_raw(
        &src.map(|element| format!("<li> {}", element))
            .collect::<Vec<String>>()
            .join("\n"),
        "ul",
        options,
    )
}

/// Generate a header of the provided level
pub fn header<T: AsRef<str>>(level: u8, content: T, options: &str) -> String {
    wrap_html_raw(content, format!("h{}", level), options)
}

pub struct HtmlTag<'a> {
    tag_type: TagType<'a>,
    class: BTreeSet<String>,
    id: Option<String>,
    options: BTreeMap<String, String>,
    inline: bool
}

pub enum TagType<'a> {
    Header(u8),
    Ul,
    Li,
    Img(&'a str),
    Video(&'a str),
    A(&'a str),
    Div, 
    Span
}

impl<'a> HtmlTag<'a>
{
    pub fn new(tag_type: TagType<'a>) -> Self {
        Self {
            tag_type,
            class: BTreeSet::new(),
            id: None,
            options: BTreeMap::new(),
            inline: false
        }
    }

    pub fn set_inline(mut self: Self, is_it: bool) -> Self { 
        self.inline = is_it; 
        self 
    }

    pub fn header(level: u8) -> Self {
        Self::new(TagType::Header(level))
    }

    pub fn div() -> Self {
        Self::new(TagType::Div)
    }

    pub fn span() -> Self {
        Self::new(TagType::Span)
            .set_inline(true)
    }

    pub fn ul() -> Self {
        Self::new(TagType::Ul)
    }

    pub fn li() -> Self {
        Self::new(TagType::Li)
    }

    pub fn a(link: &'a str) -> Self {
        Self::new(TagType::A(link))
    }

    pub fn img(src: &'a str) -> Self {
        Self::new(TagType::Img(src))
    }

    pub fn video(src: &'a str) -> Self {
        Self::new(TagType::Video(src))
    }

    pub fn with_class<T: Into<String>>(&mut self, class_name: T) -> &mut Self {
        self.class.insert(class_name.into());
        self 
    }

    pub fn with_id<T: Into<String>>(&mut self, class_name: T) -> &mut Self {
        self.class.insert(class_name.into());
        self 
    }

    pub fn with_attr<K: Into<String>, V: Into<String>>(&mut self, attr_name: K, attr_value: V) -> &mut Self { 
        self.options.insert(attr_name.into(), attr_value.into());
        self
    }

    fn format_attrs(&self) -> String {
        let mut result = "".to_string();
        for (attr_name, attr_value) in self.options.iter() {
            result.push_str(&format!("{}=\"{}\" ", attr_name, attr_value));
        }
        result
    }

    fn format_class(&self) -> String {
        let mut result = "".to_string();
        for class in self.class.iter() {
            result.push_str(&format!(" class=\"{}\" ", class));
        }
        result
    }

    fn format_id(&self) -> String {
        match &self.id {
            Some(id_name) => format!(" id=\"{}\"", id_name), 
            None => "".to_string()
        }
    }

    fn format_tag_attrs(&self) -> String {
        match self.tag_type {
            TagType::Img(src) => format!("src=\"{}\"", src),
            TagType::Video(src) => format!("src=\"{}\"", src),
            TagType::A(href) => format!("href=\"{}\"", href),
            _ => "".to_string()
        }
    }

    fn format_tag(&self) -> String {
        match self.tag_type {
            TagType::Img(_) => "img".to_string(),
            TagType::Video(_) => "video".to_string(), 
            TagType::Ul => "ul".to_string(),
            TagType::Li => "li".to_string(),
            TagType::Header(level) => format!("h{}", level),
            TagType::A(_) => "a".to_string(),
            TagType::Div => "div".to_string(),
            TagType::Span => "span".to_string(),
        }

    }

    pub fn wrap<T: std::fmt::Display>(&self, content: T) -> String {
        let attr_fmt = self.format_attrs();
        let id_fmt = self.format_id();
        let cls_fmt = self.format_class();
        let tag_fmt = self.format_tag();
        let tag_attr = self.format_tag_attrs();
        let linebreak = match self.inline {
            true => " ",
            false => "\n"
        };

        format!("<{tag} {tag_attrs} {classes}{idname}{attrs}>{linebreak}{content}{linebreak}</{tag}>", 
            tag=tag_fmt, tag_attrs=tag_attr, 
            classes=cls_fmt, idname=id_fmt, attrs=attr_fmt, 
            content=content, linebreak=linebreak
        ) 
    }

}
