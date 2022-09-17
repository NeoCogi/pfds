use crate::{HashMap, HashSet, Hashable};
use std::sync::Arc;

#[derive(Clone)]
pub struct Node<D: Clone> {
    id: usize,
    tree: Arc<Tree<D>>,
}

impl<D: Clone> Node<D> {
    pub fn data(&self) -> &D {
        self.tree.data.find(&self.id).unwrap()
    }
}

#[derive(Clone)]
struct Tree<D: Clone> {
    count: usize,
    data: HashMap<usize, D>,
    child_to_parent: HashMap<usize, usize>,
    children: HashMap<usize, HashSet<usize>>,
    roots: HashSet<usize>,
}

impl<D: Clone> Tree<D> {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            count: 0,
            data: HashMap::empty(),
            child_to_parent: HashMap::empty(),
            children: HashMap::empty(),
            roots: HashSet::empty(),
        })
    }

    pub fn add_node(&self, parent: Option<usize>, data: D) -> (Node<D>, Arc<Self>) {
        let node_id = self.count;
        let (child_to_parent, children, roots) = match parent {
            Some(p) => {
                let parent_children = self.children.find(&p).unwrap();
                let new_children = self.children.insert(p, parent_children.insert(node_id));
                (
                    self.child_to_parent.insert(node_id, p),
                    new_children,
                    self.roots.clone(),
                )
            }
            None => (
                self.child_to_parent.clone(),
                self.children.clone(),
                self.roots.insert(node_id),
            ),
        };

        let tree = Arc::new(Self {
            count: node_id + 1,
            data: self.data.insert(node_id, data),
            child_to_parent,
            children,
            roots,
        });
        let node = Node {
            id: node_id,
            tree: tree.clone(),
        };
        (node, tree)
    }

    pub fn remove_node(&self, node: &Node<D>) -> Arc<Self> {
        let data = self.data.remove(node.id);
        let node_children = self.children.find(&node.id).unwrap().to_vec();
        let children = self.children.remove(node.id);
        let mut child_to_parent = self.child_to_parent.clone();
        for c in node_children {
            child_to_parent = child_to_parent.remove(c);
        }

        let roots = {
            let parent = child_to_parent.find(&node.id);
            match parent {
                Some(p) => {
                    // remove self from the child to parent
                    child_to_parent = child_to_parent.remove(node.id);
                    self.roots.clone()
                }
                None => self.roots.remove(node.id),
            }
        };

        Arc::new(Self {
            count: self.count,
            data,
            children,
            child_to_parent,
            roots,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::hashset::*;
    use crate::tree::*;

    static mut SEED: i64 = 777;

    fn rand() -> i32 {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32
        }
    }

    #[test]
    fn add_children() {}
}
