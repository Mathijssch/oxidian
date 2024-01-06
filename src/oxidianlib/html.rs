/// General utility function to wrap a given string into a html tag. 
pub fn wrap_html_raw<T: AsRef<str>, U: AsRef<str>, V: AsRef<str>> (content: T, tag: U, options: V) -> String {
    return format!("<{} {}>{}</{}>", tag.as_ref(), options.as_ref(), content.as_ref(), tag.as_ref()); 
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
pub fn link(dst: &std::path::Path, text: &str, options: &str) -> String {
    wrap_html_raw(text, "a", &format!("href={} {}", dst.to_string_lossy(), options))
}


///Generate and unordered list of elements that can be displayed.
pub fn ul<T: std::fmt::Display, U: Iterator<Item = T>> (src: U, options: &str) -> String {
    wrap_html_raw(
        &src.map(|element| format!("<li> {}", element))
        .collect::<Vec<String>>()
        .join("\n")
        , "ul", options
    )
}
