use log::debug;
use crate::utils::formatting::link_to_html;
use crate::core::html;
use crate::utils::utils;
use crate::components::link::Link;
use crate::utils::filesys;
use std::fs::File;
use std::io::Write;
use std::collections::{BTreeMap, BTreeSet};
use crate::utils::constants::TAG_DIR;
use std::path::{PathBuf,Path};


#[derive(PartialEq, Eq, Debug)]
pub struct Tree {
    pub name: String,
    pub children: BTreeMap<String, Tree>,
    pub contents: BTreeSet<Link>,
}

impl Tree {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Tree {
            name: name.into(),
            children: BTreeMap::new(),
            contents: BTreeSet::new(),
        }
    }


    /// Add a tree with a single branch, adding the links to the leaf element.
    pub fn from_iter_payload<I, It, S>(branch: I, links: It) -> Option<Self> 
    where I: IntoIterator<Item=S>, 
          S: Into<String>,
          It: IntoIterator<Item=Link> + Clone 
    {
        let mut branch_iter = branch.into_iter().peekable();
        let mut tree: Tree; 
        if let Some(first_element) = branch_iter.next() {
            // Initialize the tree.
            tree = Self::new(first_element);
        } else {
            // Iterator is empty.
            return None;
        }

        // NOTE
        // The way the links are added to the leaf node is a bit clunky but this is to avoid
        // interior mutability. Once the leaf subtree is added to the tree, the ownership is moved
        // there, and we can no longer access our reference to it. An alternative way to do things
        // would be to implement a way to add links to a descendant of the tree with a given
        // sequence of names, but this would require another set of iterations through the tree.
        
        if branch_iter.peek().is_none() {
            // No more elements left. 
            for link in links { 
                tree.contents.insert(link);
            }
            return Some(tree);
        }

        let mut subtree: Tree; 
        while let Some(node) = branch_iter.next() { 
            subtree = Tree::new(node);
            if branch_iter.peek().is_none() {
                // last iteration, first add the links to avoid interior mutability.
                for link in links.clone() { 
                    subtree.contents.insert(link);
                }
            }
            tree.add_child(subtree);
            }
        Some(tree)
    }


    /// Make a tree consisting of a single branch, where successive elements in the iter are set as
    /// child of the previous. If the given iterator is empty, then no tree is returned.
    #[allow(dead_code)]
    pub fn from_iter<I, S>(branch: I) -> Option<Self> 
    where I: IntoIterator<Item=S>, 
          S: Into<String>
    {Self::from_iter_payload(branch, vec![])}

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    #[allow(dead_code)]
    pub fn add_link(&mut self, reference: Link) {
        self.contents.insert(reference);
    }

    pub fn get_count_recursive(&self) -> usize {
        let mut result = self.contents.len();
        for subtree in self.children.values() {
            result += subtree.get_count_recursive();
        }
        result
    }
    
    /// Get all the items in this tree recursively (including the items in the subtrees) 
    /// TODO - Figure out how to do this with iterators.
    #[allow(dead_code)]
    pub fn get_contents_recursive(&self) -> Vec<&Link> {
        let mut result: Vec<&Link> = self.contents.iter().collect();
        for subtree in self.children.values() {
            let mut subtree_content = subtree.get_contents_recursive(); 
            result.append(&mut subtree_content)
        }
        result
    }

    /// Add a child to the tree and return a reference to that child.
    pub fn add_child<'a>(&'a mut self, child: Tree) {
        let name = child.name.to_owned();
        if let Some(previous_child) = self.children.insert(name.clone(), child) {
            // Re-add the already existing grandchildren under the same name.
            let new_children = self.children.get_mut(&name);
            if let Some(subtree) = new_children {
                // Copy over the contents of the replaced child.
                subtree.contents = subtree.contents.union(&previous_child.contents)
                    .cloned().collect(); // TO-DO: Get rid of the cloning here??
                for old_grandchild in previous_child.children {
                    subtree.add_child(old_grandchild.1);
                }
            }
        }
    }
}

/// Formatting stuff
impl Tree {
 
    pub fn to_html(&self) -> String {
        self.to_html_inner(false, &Path::new(TAG_DIR))
    }


    fn to_html_inner(&self, is_nested: bool, base_path: &Path) -> String {
        let mut options = "".to_string();
        if !is_nested {
            options.push_str("class=\"nav_tag_list\" id=\"nav_tag_list\"");
        }

        let mut child_basepath = base_path.to_owned();
        let mut ego_entry = self.name.clone();

        if is_nested { 
            let curr_page_filename = utils::generate_tag_page_name(&self.name);
            let curr_page_path = base_path.join(curr_page_filename);
            child_basepath.push(&self.name);
            ego_entry = html::link(utils::prepend_slash(&curr_page_path).as_path(), 
                &self.name, "") + 
                &html::HtmlTag::span()
                     .with_class("tag-count")
                     .with_attr("style", "float: right")
                     .wrap(self.get_count_recursive());
        }

        if !self.is_leaf() {
            ego_entry = html::HtmlTag::summary().wrap(ego_entry);
            // expand recursively 
            let sublist = self.children                    
                    .values().map(|subtree| subtree.to_html_inner(true, &child_basepath));
            ego_entry.push_str(&html::ul(sublist, &options));
            ego_entry = html::HtmlTag::details().wrap(ego_entry);
        }
        ego_entry
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

        html_content
    }

    ///Build the index of a given tree of tags (recursively). 
    ///
    ///Arguments: 
    ///  - `output_path`: path to the directory where the website is generated. 
    ///  - `base_path`: directory where the tag pages are generated
    ///  - `template`: html template to build a webpage.
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


#[cfg(test)]
mod tests {

    use std::assert_eq;

    use super::Tree;


    fn create_first_tree() -> Tree {
        let mut root = Tree::new("root");

        // first tree: a/b/c
        let mut a = Tree::new("a");
        let mut b = Tree::new("b"); 
        let c = Tree::new("c");

        b.add_child(c);
        a.add_child(b);
        root.add_child(a);

        //second subtree tree: a/d
        let mut a2 = Tree::new("a");
        let d = Tree::new("d"); 

        a2.add_child(d);

        root.add_child(a2);
        root
    }
    

    fn create_second_tree() -> Tree {
        let mut root = Tree::new("root");

        // first tree: a/b/c
        let mut a = Tree::new("a");
        let mut b = Tree::new("b"); 
        let c = Tree::new("c");

        b.add_child(c);
        a.add_child(b);

        //second subtree tree: a/d
        let d = Tree::new("d"); 
        a.add_child(d);
        root.add_child(a);
        root
    }

    #[test]
    fn add_existing_child() {
        let tree1 = create_first_tree();
        let tree2 = create_second_tree();
        assert_eq!(tree1, tree2);
    }

}

