use log::debug;
use slugify::slugify;
use super::{html, utils, filesys};
use std::fs::File;
use std::io::Write;
use super::link::{Link, LinkType, FileType};
use super::obs_tags::Tag;
use super::filesys::slugify_path;
use super::tag_tree::Tree;
use std::path::{PathBuf,Path};
use super::utils::prepend_slash;

fn md_link(text: &str, target: &str) -> String {
    format!("[{}]({})", text, target).to_string()
}


pub fn tag_to_md(tag: &Tag) -> String {
    return html::wrap_html_raw(&tag.tag_path, "span", "class=\"tag\"");
}

pub fn link_to_html(link: &Link) -> String { render_link(link, true) }
pub fn link_to_md(link: &Link) -> String { render_link(link, false) }

fn render_link_aux(tg: &str, text: &str, to_html: bool, classes: Option<&Vec<&str>>) -> String {
    match to_html {
        true => { let mut a_tag = html::HtmlTag::a(tg); 
            if let Some(classlist) = classes { 
                for class in classlist {
                    a_tag.with_class(*class);
                };
            };
            a_tag.wrap(text)
        },
        false => md_link(text, tg)
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
            let mut target_abs = slugify_path(&link.target, Some("html")).unwrap()
                                    .to_string_lossy()
                                    .to_string();
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
                return render_link_aux(&target_abs, &link_text, to_html, Some(&vec!["broken"]));
            } else {
                return render_link_aux(&target_abs, &link_text, to_html, None);
            }
        },
        LinkType::Internal => {
            let mut target_abs = "".to_string();
            if let Some(subtarget) = &link.subtarget {
                target_abs.push_str("#");
                target_abs.push_str(subtarget);
            } 
            return render_link_aux(&target_abs, &link_text, to_html, None);
        },
        LinkType::External => {return md_link(&link_text, &link_target_str);},
        LinkType::Attachment(filetype) => {
            let target_rel  = slugify_path(&link.target, None).unwrap();
            let target_file = prepend_slash(&target_rel)
                .to_string_lossy()
                .to_string();
            match filetype {
                FileType::Image => {return html::img_tag(&target_file)},
                FileType::Video => {return html::video_tag(&target_file);},
                _ => { return render_link_aux(&link_target_str, &link_text, to_html, None); }
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

        let mut child_basepath = base_path.to_owned();
        let mut ego_entry = self.name.clone();

        if is_nested { 
            let curr_page_filename = utils::generate_tag_page_name(&self.name);
            let curr_page_path = base_path.join(curr_page_filename);
            child_basepath.push(&self.name);
            ego_entry = html::link(utils::prepend_slash(&curr_page_path).as_path(), &self.name, "");
        }

        if !self.is_leaf() {
            // expand recursively 
            let sublist = self.children                    
                    .values().map(|subtree| subtree.to_html_inner(true, &child_basepath));
            ego_entry.push_str(&html::ul(sublist, &options));
        }
        return ego_entry
    }

    fn flush_letter_list(html_content: &mut String, li_notes_per_letter: &mut String) {
        html_content.push_str(
            &html::HtmlTag::div().with_class("tag_list_wrapper")
                .wrap(
                &html::HtmlTag::ul()
                .with_class("tag_list")
                .wrap( &li_notes_per_letter )
            )
        );
        *li_notes_per_letter = "".to_string();
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

        // FIXED -- sorting is guaranteed by BTreeSet
        //let mut sorted_links: Vec<&Link> = self.contents.iter().collect();
        //    sorted_links.sort_unstable_by_key(|link| link.link_text());

        let mut letter: Option<char> = None;
        let mut li_notes_per_letter = "".to_string();
        for link in &self.contents {
            let new_initial = match letter {
                Some(l) => l.to_lowercase().ne(utils::initial(link.link_text()).to_lowercase()),
                None    => true
            };
            if new_initial {
                if letter.is_some() { // Already covered a letter, so flush the list of notes.
                    Self::flush_letter_list(&mut html_content, &mut li_notes_per_letter);
                }
                let curr_initial = utils::initial(link.link_text());
                letter = Some(curr_initial);
                let h2 = html::HtmlTag::header(2).wrap(curr_initial.to_uppercase());
                html_content.push_str(&h2);
            }
            
            debug!("Adding link       {:?} of type {:?}", link, link.link_type());
            debug!("Gets converted to {:?}", link_to_html(&link));
            li_notes_per_letter.push_str(
                &html::HtmlTag::li().wrap(
                    link_to_html(&link)
                )
            );
        }
        if li_notes_per_letter.len() > 0 { // Flush whatever is left.
            Self::flush_letter_list(&mut html_content, &mut li_notes_per_letter);
        }

        return html_content;
    }

    pub fn build_index_pages(
        &self, output_path: &Path, 
        base_path: &Path, template: &str
    ) -> std::io::Result<()> {
        // Generate the html for its own page.
        let parent_tags = vec![];
        for child in self.children.values() { 
            child.inner_build_index_pages(output_path, base_path, &parent_tags, template)?;
        }
        Ok(())
    }

    fn prepare_directory(base_path: &Path, parent_tags: &Vec<&Link>) -> std::io::Result<PathBuf> {
        //let mut directory = base_path.to_owned(); 
        //for tag in parent_tags { 
        //    directory = directory.join(tag.link_text());
        //}
        let mut dir = PathBuf::new();
        if let Some(parent) = parent_tags.last() {
            if let Some(parents_subpath) = parent.target.parent() {
                dir.push(parents_subpath
                    .strip_prefix(utils::prepend_slash(base_path))
                    .unwrap()
                );
            } 
            dir.push(parent.link_text());
        }
        Ok(dir)
    }

    pub fn inner_build_index_pages(
        &self,
        output_path: &Path,
        base_path: &Path, 
        inner_tags: &Vec<&Link>,
        template: &str
    ) -> std::io::Result<()> 
    {

        let rel_dir = Self::prepare_directory(&base_path, &inner_tags)?;
 
        // Generate the html for its own page. 
        let html_content = self.to_html_index(inner_tags);

        let curr_page_filename = utils::generate_tag_page_name(&self.name);
        let relative_page_path = base_path.join(rel_dir.join(curr_page_filename));
        //let absolute_page_dir  = output_path.join(&rel_dir);
        let absolute_page_path = output_path.join(&relative_page_path); 

        //info!("Relative path {:?}", relative_page_path);
        if let Some(parent_dir) = absolute_page_path.parent() {
            debug!("Making directory {:?}", parent_dir);
            filesys::create_dir_if_not_exists(&parent_dir)?;
        }
        //info!("Now creating file {:?}", absolute_page_path);
        let file = File::create(&absolute_page_path)?;
        debug!("Writing tag page {:?}", absolute_page_path);
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
        let link_to_self = Link::new(self.name.clone(), utils::prepend_slash(&relative_page_path));
        inner_tags.push(&link_to_self);
        for child in self.children.values() {
            child.inner_build_index_pages(
                &output_path,
                &base_path, 
                &inner_tags, 
                template
            )?;
        }
        Ok(())
    }
}


impl Link {
    pub fn to_html(&self) -> String { link_to_html(&self) }
    pub fn to_md(&self) -> String { link_to_md(&self) }
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
        let slug_name = slugify_path(Path::new("path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("path_to_note|alias", false).unwrap(),
            expected_output: format!("[alias](/{})", slug_name).to_string()
        }
    }
    fn create_note_in_dir_link() -> TestCase {
        let slug_name = slugify_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("subdir/path_to_note", false).unwrap(),
            expected_output: format!("[subdir/path_to_note](/{})", slug_name).to_string()
        }
    }
    fn create_note_in_dir_sublink() -> TestCase {
        let slug_name = slugify_path(Path::new("subdir/path_to_note"), Some("html"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        TestCase { 
            input_link: Link::from_obsidian_link("subdir/path_to_note#heading|alias", false).unwrap(),
            expected_output: format!("[alias](/{}#heading)", slug_name).to_string()
        }
    }
    fn create_image_link() -> TestCase {
        let slug_name = slugify_path(Path::new("path_to_image.png"), None)
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

