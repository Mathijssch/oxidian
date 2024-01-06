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

