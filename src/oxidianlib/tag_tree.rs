use super::link::Link;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Debug)]
pub struct Tree {
    pub name: String,
    pub children: HashMap<String, Tree>,
    pub contents: HashSet<Link>,
}

impl Tree {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Tree {
            name: name.into(),
            children: HashMap::new(),
            contents: HashSet::new(),
        }
    }

    pub fn from_iter_payload<I, It, S>(branch: I, links: It) -> Option<Self> 
    where I: IntoIterator<Item=S>, 
          S: Into<String>,
          It: IntoIterator<Item=Link> 
    {
        let mut branch_iter = branch.into_iter();
        let mut tree: Tree; 
        if let Some(first_element) = branch_iter.next() {
            // Initialize the tree.
            tree = Self::new(first_element);
        } else {
            // Iterator is empty.
            return None;
        }
        let mut subtree: Tree; 
        for node in branch_iter {
            subtree = Tree::new(node);
            tree.add_child(subtree);
        }
        for link in links { 
            subtree.add_link(link);
        }
        Some(tree)
    }


    /// Make a tree consisting of a single branch, where successive elements in the iter are set as
    /// child of the previous. If the given iterator is empty, then no tree is returned.
    pub fn from_iter<I, S>(branch: I) -> Option<Self> 
    where I: IntoIterator<Item=S>, 
          S: Into<String>
    {Self::from_iter_payload(branch, vec![])}

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn add_link(&mut self, reference: Link) {
        self.contents.insert(reference);
    }
    
    /// Get all the items in this tree recursively (including the items in the subtrees) 
    /// TODO - Figure out how to do this with iterators.
    pub fn get_contents_recursive(&self) -> Vec<&Link> {
        let mut result: Vec<&Link> = self.contents.iter().collect();
        for subtree in self.children.values() {
            let mut subtree_content = subtree.get_contents_recursive(); 
            result.append(&mut subtree_content)
        }
        result
    }

    pub fn add_child(&mut self, child: Tree) {
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

