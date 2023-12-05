use super::html;
use super::link::{Link, LinkType, FileType};
use super::filesys::convert_path;
use std::path::Path;
use super::utils::prepend_slash;

fn md_link(text: &str, target: &str) -> String {
    format!("[{}]({})", text, target).to_string()
}


pub fn link_to_md(link: &Link) -> String {

    //println!("Converting link {:?}", link);
    //println!("Link has type {:?}", link.link_type());

    let link_target_str = link.target.to_string_lossy().to_string();
    let link_text = link.alias.as_ref().unwrap_or_else(|| &link_target_str );

    match link.link_type() {
        LinkType::Note => {
            // Link to note should point to html page.
            let target_rel = convert_path(&Path::new(&link.target), Some("html")).unwrap();
            let mut target_abs = prepend_slash(&target_rel)
                .to_string_lossy()
                .to_string();
            if let Some(subtarget) = &link.subtarget {
                target_abs.push_str("#");
                target_abs.push_str(subtarget);
            }
            return md_link(&link_text, &target_abs);
        },
        LinkType::External => {return md_link(&link_text, &link_target_str);},
        LinkType::Attachment(filetype) => {
            let target_rel = convert_path(&link.target, None).unwrap();
            let target_file = prepend_slash(&target_rel)
                .to_string_lossy()
                .to_string();
            match filetype {
                FileType::Image => {return html::img_tag(&target_file)},
                FileType::Video => {return html::video_tag(&target_file);},
                _ => { return md_link(&link_text, &link_target_str) }
            }
        }, 
        _ => {return md_link(&link_text, &link_target_str);}
    };
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::Link;
    use std::assert_eq;

    struct TestCase { 
        input_link: Link,
        expected_output: String 
    }

    fn create_note_link() -> TestCase {
        let slug_name = convert_path(Path::new("path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("path_to_note|alias", false).unwrap(),
            expected_output: format!("[alias]({})", slug_name).to_string()
        }
    }
    fn create_note_in_dir_link() -> TestCase {
        let slug_name = convert_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("subdir/path_to_note", false).unwrap(),
            expected_output: format!("[subdir/path_to_note]({})", slug_name).to_string()
        }
    }
    fn create_note_in_dir_sublink() -> TestCase {
        let slug_name = convert_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("subdir/path_to_note#heading|alias", false).unwrap(),
            expected_output: format!("[alias]({}#heading)", slug_name).to_string()
        }
    }
    fn create_image_link() -> TestCase {
        let slug_name = convert_path(Path::new("path_to_image.png"), None)
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase {
            input_link: Link::from_obsidian_link("path_to_image.png", true).unwrap(),
            expected_output: format!("<img src=\"{}\"></img>", slug_name).to_string()
        }
    }
    
    //fn create_svg_link() -> Link {Link::from_obsidian_link("path_to_image.svg", true).unwrap()}
    //fn create_video_link() -> Link {Link::from_obsidian_link("path_to_image.mp4", true).unwrap()}

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

