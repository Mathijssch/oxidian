use super::link::Link;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub struct Tree {
    pub name: String,
    pub children: HashMap<String, Tree>,
    pub contents: Vec<Link>,
}

impl Tree {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Tree {
            name: name.into(),
            children: HashMap::new(),
            contents: Vec::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn add_link(&mut self, reference: Link) {
        self.contents.push(reference);
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

