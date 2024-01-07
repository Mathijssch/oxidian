use log::info;
use super::{html, utils, filesys, link};
use std::fs::File;
use std::io::Write;
use super::link::{Link, LinkType, FileType};
use super::obs_tags::Tag;
use super::filesys::convert_path;
use super::tag_tree::Tree;
use std::path::Path;
use super::utils::prepend_slash;

fn md_link(text: &str, target: &str) -> String {
    format!("[{}]({})", text, target).to_string()
}


pub fn tag_to_md(tag: &Tag) -> String {
    return html::wrap_html_raw(&tag.tag_path, "span", "class=\"tag\"");
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
            } return md_link(&link_text, &target_abs); },
        LinkType::Internal => {
            let mut target_abs = "".to_string();
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
        } 
        //_ => {return md_link(&link_text, &link_target_str);}
    };
}


impl Tree {
 
    pub fn to_html<T: AsRef<Path>> (&self, reference_path: T) -> String {
        self.to_html_inner(false, reference_path.as_ref())
    }


    fn to_html_inner(&self, is_nested: bool, base_path: &Path) -> String {
        let mut options = "".to_string();
        if !is_nested {
            options.push_str("class=\"\"");
        }

        let curr_page_filename = utils::generate_tag_page_name(&self.name);
        let curr_page_path = base_path.join(curr_page_filename);
        let child_basepath = base_path.join(&self.name);

        let ego_entry = html::link(&curr_page_path, &self.name, "");
        let iter_items = vec![ego_entry].into_iter()
            .chain(
                self.children.values().map(|subtree| subtree.to_html_inner(true, &child_basepath))
        );
        html::ul(iter_items, &options)
    }
    
    // Generate the html for the index page
    fn to_html_index(&self, parent_tags: &Vec<&Link>) -> String {
        let mut html_content = html::HtmlTag::header(1).wrap(format!("Index of {}", self.name));
        
        // Links to subtags
        { 
            let links_to_subtags = parent_tags.iter()
                .map(|link| html::link(&link.target, &link.link_text(), ""))
                .chain(vec![self.name.to_string()].into_iter()); 

            let breadcrumbs = html::ul( links_to_subtags, "class=\"breadcrumbs\"");
            html_content.push_str(&breadcrumbs);
        }

        let mut sorted_links: Vec<&Link> = self.contents.iter().collect();
        sorted_links.sort_unstable_by_key(|link| link.link_text());

        let mut letter: Option<char> = None;
        let mut li_notes_per_letter = "".to_string();
        for link in sorted_links {
            let new_initial = match letter {
                Some(l) => l.to_lowercase().ne(utils::initial(link.link_text()).to_lowercase()),
                None    => true
            };
            if new_initial {
                if letter.is_some() { // Already covered a letter, so flush the list of notes.
                    html_content.push_str(
                        &html::HtmlTag::div().with_class("tag_list_wrapper")
                            .wrap(
                            &html::HtmlTag::ul()
                            .with_class("tag_list")
                            .wrap( li_notes_per_letter )
                        )
                    );
                    li_notes_per_letter = "".to_string();
                }
                let curr_initial = utils::initial(link.link_text());
                letter = Some(curr_initial);
                let h2 = html::HtmlTag::header(2).wrap(curr_initial);
                html_content.push_str(&h2);
            }
            
            li_notes_per_letter.push_str(
                &html::HtmlTag::li().wrap(
                    &html::link(&link.target, &link.link_text(), "")
                )
            );
        }
        return html_content;
    }

    pub fn build_index_pages(&self, base_path: &Path, template: &str) -> std::io::Result<()> {
        // Generate the html for its own page.
        let parent_tags = vec![];
        for child in self.children.values() { 
            child.inner_build_index_pages(base_path, &parent_tags, template)?;
        }
        Ok(())
    }

    pub fn inner_build_index_pages(
        &self,
        base_path: &Path, 
        inner_tags: &Vec<&Link>,
        template: &str
    ) -> std::io::Result<()> 
    {
        let mut directory = base_path.to_owned(); 
        for tag in inner_tags { 
            directory = directory.join(tag.link_text());
        }
        filesys::create_dir_if_not_exists(&directory)?; 
        
        // Generate the html for its own page. 
        let html_content = self.to_html_index(&inner_tags);

        let curr_page_filename = utils::generate_tag_page_name(&self.name);
        let curr_page_path = base_path.join(curr_page_filename);
        //let child_basepath = base_path.join(&self.name);

        let file = File::create(&curr_page_path)?;
        info!("Writing tag page {:?}", curr_page_path);
        let mut writer = std::io::BufWriter::new(file);
        let parent_tag_names: Vec<String> = inner_tags.iter().map(
            |t| t.link_text()
        ).collect();
        // Title and header 
        let title = format!("Tag - {} / {}", parent_tag_names.join(" / "), self.name);
        let html = template.replace("{{content}}", &html_content)
            .replace("{{backlinks}}", "")
            .replace("{{title}}", &title);

        writer.write_all(html.as_bytes())?;

        let mut inner_tags = inner_tags.clone();
        let link_to_self = Link::new(self.name.clone(), curr_page_path);
        inner_tags.push(&link_to_self);
        for child in self.children.values() {
            child.inner_build_index_pages(
                base_path, 
                &inner_tags, 
                template
            )?;
        }
        Ok(())
    }
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
            expected_output: format!("[alias](/{})", slug_name).to_string()
        }
    }
    fn create_note_in_dir_link() -> TestCase {
        let slug_name = convert_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("subdir/path_to_note", false).unwrap(),
            expected_output: format!("[subdir/path_to_note](/{})", slug_name).to_string()
        }
    }
    fn create_note_in_dir_sublink() -> TestCase {
        let slug_name = convert_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("subdir/path_to_note#heading|alias", false).unwrap(),
            expected_output: format!("[alias](/{}#heading)", slug_name).to_string()
        }
    }
    fn create_image_link() -> TestCase {
        let slug_name = convert_path(Path::new("path_to_image.png"), None)
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase {
            input_link: Link::from_obsidian_link("path_to_image.png", true).unwrap(),
            expected_output: format!("<img src=\"/{}\"></img>", slug_name).to_string()
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

