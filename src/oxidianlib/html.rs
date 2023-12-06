pub fn wrap_html_raw(content: &str, tag: &str, options: &str) -> String {
    return format!("<{} {}>{}</{}>", tag, options, content, tag); 
}

pub fn video_tag(src: &str) -> String {
    wrap_html_raw("", "video", &format!("src=\"{}\"", src))
}

pub fn img_tag(src: &str) -> String {
    wrap_html_raw("", "img", &format!("src=\"{}\"", src))
}

pub fn link(dst: &std::path::Path, text: &str, options: &str) -> String {
    wrap_html_raw(text, "a", &format!("href={} {}", dst.to_string_lossy(), options))
}

pub fn ul<T: std::fmt::Display, U: Iterator<Item = T>> (src: U, options: &str) -> String {
    wrap_html_raw(
        &src.map(|element| format!("<li> {}", element))
        .collect::<Vec<String>>()
        .join("\n")
        , "ul", options
    )
}
