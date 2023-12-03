use super::html;
use super::link::{Link, LinkType, FileType};
use super::filesys::convert_path;
use std::path::Path;

fn md_link(text: &str, target: &str) -> String {
    format!("[{}]({})", text, target).to_string()
}

pub fn link_to_md(link: &Link) -> String {

    let link_target_str = link.target.to_string_lossy().to_string();
    let link_text = link.alias.as_ref().unwrap_or_else(|| &link_target_str );

    match link.link_type() {
        LinkType::Note => {
            // Link to note should point to html page.
            let mut target = convert_path(&Path::new(&link.target)).unwrap()
                .to_string_lossy()
                .to_string();
            if let Some(subtarget) = link.subtarget {
                target = format!("{}#{}", target, subtarget);
            }
            return md_link(&link_text, &target);
        },
        LinkType::External => {return md_link(&link_text, &link_target_str);},
        LinkType::Attachment(filetype) => {
            match filetype {
                FileType::Image => {return html::img_tag(&link_target_str)},
                FileType::Video => {return html::video_tag(&link_target_str);},
                _ => { return md_link(&link_text, &link_target_str) }

            }
        }, 
        _ => {return md_link(&link_text, &link_target_str);}
    };
}


