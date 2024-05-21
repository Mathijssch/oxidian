use super::utils;
use crate::core::html;
use slugify::slugify;

use crate::utils::filesys::slugify_path;
use super::utils::prepend_slash;
use crate::components::link::{FileType, Link, LinkType};
use crate::obsidian::tags::Tag;

fn md_link(text: &str, target: &str) -> String {
    format!("[{}]({})", text, target).to_string()
}

pub fn tag_to_md(tag: &Tag) -> String {
    html::HtmlTag::span()
        .with_class("tag")
        .wrap(html::HtmlTag::a(&utils::format_tag_path(&tag)).wrap(&tag.tag_path))
}

pub fn link_to_html(link: &Link) -> String {
    render_link(link, true)
}
pub fn link_to_md(link: &Link) -> String {
    render_link(link, false)
}

fn render_link_aux(tg: &str, text: &str, to_html: bool, classes: Option<&Vec<&str>>) -> String {
    match to_html {
        true => {
            let mut a_tag = html::HtmlTag::a(tg);
            if let Some(classlist) = classes {
                for class in classlist {
                    a_tag.with_class(*class);
                }
            };
            a_tag.wrap(text)
        }
        false => md_link(text, tg),
    }
}

/// Render link to string
fn render_link(link: &Link, to_html: bool) -> String {
    let link_target_str = link.target.to_string_lossy().to_string();
    let link_text = link.link_text();

    //let mut target_abs = "".to_string();
    match link.link_type() {
        LinkType::Note => {
            // Link to note should point to html page.
            let mut target_abs = slugify_path(&link.target, Some("html"))
                .unwrap()
                .to_string_lossy()
                .to_string();
            //info!("Sluggified path {:?} to {:?}", link.target, target_abs);
            //let mut target_abs = prepend_slash(&target_rel)
            //    .to_string_lossy()
            //    .to_string();
            if let Some(subtarget) = &link.subtarget {
                target_abs.push_str("#");
                if subtarget.starts_with('^') {
                    target_abs.push_str(&subtarget['^'.len_utf8()..]);
                } else {
                    target_abs.push_str(&slugify!(subtarget));
                }
            }
            if link.broken {
                render_link_aux(&target_abs, &link_text, true, Some(&vec!["broken"]))
            } else {
                render_link_aux(&target_abs, &link_text, to_html, None)
            }
        }
        LinkType::Internal => {
            let mut target_abs = "".to_string();
            if let Some(subtarget) = &link.subtarget {
                target_abs.push_str("#");
                target_abs.push_str(subtarget);
            }
            render_link_aux(&target_abs, &link_text, to_html, None)
        }
        LinkType::External => md_link(&link_text, &link_target_str),
        LinkType::Attachment(filetype) => {
            let target_rel = slugify_path(&link.target, None).unwrap();
            let target_file = prepend_slash(&target_rel).to_string_lossy().to_string();

            let mut tag = match filetype {
                FileType::Image => html::HtmlTag::img(&target_file),
                FileType::Video => {let mut video_tag = html::HtmlTag::video(&target_file);
                                    video_tag.with_attr("controls", "");
                                    video_tag
                                    }
                _ => { return render_link_aux(&link_target_str, &link_text, to_html, None); },
            };

            if let Some(dims) = link.parse_dims() {
                tag.with_attr("width", dims.width);
                if let Some(h) = dims.height {
                    tag.with_attr("height", h);
                }
            }
            tag.wrap("")
        } //_ => {return md_link(&link_text, &link_target_str);}
    }
}

impl Link {
    pub fn to_html(&self) -> String {
        link_to_html(&self)
    }
    pub fn to_md(&self) -> String {
        link_to_md(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_eq;
    use std::path::Path;

    struct TestCase {
        input_link: Link,
        expected_output: String,
    }

    fn create_note_link() -> TestCase {
        let slug_name = slugify_path(Path::new("path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase {
            input_link: Link::from_obsidian_link("path_to_note|alias", false).unwrap(),
            expected_output: format!("[alias]({})", slug_name).to_string(),
        }
    }
    fn create_note_in_dir_link() -> TestCase {
        let slug_name = slugify_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase {
            input_link: Link::from_obsidian_link("subdir/path_to_note", false).unwrap(),
            expected_output: format!("[path_to_note]({})", slug_name).to_string(),
        }
    }
    fn create_note_in_dir_sublink() -> TestCase {
        let slug_name = slugify_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase {
            input_link: Link::from_obsidian_link("subdir/path_to_note#heading|alias", false)
                .unwrap(),
            expected_output: format!("[alias]({}#heading)", slug_name).to_string(),
        }
    }
    fn create_image_link() -> TestCase {
        let slug_name = slugify_path(Path::new("path_to_image.png"), None)
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase {
            input_link: Link::from_obsidian_link("path_to_image.png", true).unwrap(),
            expected_output: format!("<img src=\"{}\">  </img>", slug_name).to_string(),
        }
    }

    fn basic_test(case: &TestCase) {
        let link_html = link_to_md(&case.input_link);
        assert_eq!(link_html, case.expected_output);
    }

    #[test]
    fn test_regular_note_link() {
        let test_case = create_note_link();
        basic_test(&test_case);
    }

    #[test]
    fn test_nested_note_link() {
        let test_case = create_note_in_dir_link();
        basic_test(&test_case);
    }

    #[test]
    fn test_image_link() {
        let test_case = create_image_link();
        basic_test(&test_case);
    }

    #[test]
    fn test_note_in_dir_sublink() {
        let test_case = create_note_in_dir_sublink();
        basic_test(&test_case);
    }
}
