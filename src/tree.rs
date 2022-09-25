use crate::{HashMap, HashSet, Hashable};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone)]
pub struct Node<D: Clone> {
    id: usize,
    tree: Arc<TreeIntern<D>>,
}

#[derive(Clone)]
pub struct AddChildResult<D: Clone> {
    node: Node<D>,
    child: Node<D>,
    tree: Tree<D>,
}

impl<D: Clone> AddChildResult<D> {
    pub fn node(&self) -> Node<D> {
        self.node.clone()
    }
    pub fn child(&self) -> Node<D> {
        self.child.clone()
    }
    pub fn tree(&self) -> Tree<D> {
        self.tree.clone()
    }
}

impl<D: Clone> Node<D> {
    pub fn data(&self) -> &D {
        self.tree.data.find(&self.id).unwrap()
    }

    pub fn tree(&self) -> Tree<D> {
        Tree {
            tree: self.tree.clone(),
        }
    }

    pub fn parent(&self) -> Option<Node<D>> {
        self.tree.child_to_parent.find(&self.id).map(|parent| Node {
            id: *parent,
            tree: self.tree.clone(),
        })
    }

    pub fn iter_children<'a>(&self) -> Iter<'a, D> {
        Iter {
            tree: self.tree.clone(),
            current: self.tree.children.find(&self.id).unwrap().iter(),
            _phantom: PhantomData::default(),
        }
    }

    pub fn add_child(&self, data: D) -> AddChildResult<D> {
        let (node, child, tree) = self.tree.add_node(Some(self.id), data);
        AddChildResult {
            node: node.unwrap(),
            child,
            tree: Tree { tree },
        }
    }
}

#[derive(Clone)]
struct TreeIntern<D: Clone> {
    count: usize,
    data: HashMap<usize, D>,
    child_to_parent: HashMap<usize, usize>,
    children: HashMap<usize, HashSet<usize>>,
    roots: HashSet<usize>,
}

impl<D: Clone> TreeIntern<D> {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            count: 0,
            data: HashMap::empty(),
            child_to_parent: HashMap::empty(),
            children: HashMap::empty(),
            roots: HashSet::empty(),
        })
    }

    pub fn add_node(
        &self,
        parent: Option<usize>,
        data: D,
    ) -> (
        Option<Node<D>>, /* node */
        Node<D>,         /* child */
        Arc<Self>,
    ) {
        let node_id = self.count;
        let (child_to_parent, children, roots) = match parent {
            Some(p) => {
                let parent_children = match self.children.find(&p) {
                    Some(pc) => pc.clone(),
                    None => HashSet::empty(),
                };
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

        let parent = parent.map(|id| Node {
            id,
            tree: tree.clone(),
        });

        let child = Node {
            id: node_id,
            tree: tree.clone(),
        };

        (parent, child, tree)
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

pub struct Iter<'a, E: Clone> {
    tree: Arc<TreeIntern<E>>,
    current: crate::hashset::Iter<'a, usize>,
    _phantom: PhantomData<&'a E>,
}

impl<'a, E: Clone> std::iter::Iterator for Iter<'a, E> {
    type Item = Node<E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.next() {
            Some(id) => Some(Node {
                id,
                tree: self.tree.clone(),
            }),
            None => None,
        }
    }
}

#[derive(Clone)]
pub struct Tree<D: Clone> {
    tree: Arc<TreeIntern<D>>,
}

impl<D: Clone> Tree<D> {
    pub fn empty() -> Self {
        Self {
            tree: TreeIntern::empty(),
        }
    }

    pub fn add_root_node(&self, data: D) -> (Node<D>, Self) {
        let (_parent, child, tree) = self.tree.add_node(None, data);
        (child, Self { tree })
    }

    pub fn remove_root_node(&self, node: Node<D>) -> Self {
        Self {
            tree: self.tree.remove_node(&node),
        }
    }

    pub fn roots<'a>(&self) -> Iter<'a, D> {
        Iter {
            tree: self.tree.clone(),
            current: self.tree.roots.iter(),
            _phantom: PhantomData::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::*;

    static mut SEED: i64 = 777;

    fn rand() -> i32 {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32
        }
    }

    #[test]
    fn add_roots() {
        let mut tree = Tree::empty();
        for i in 0..128 {
            let (_, t) = tree.add_root_node(i);
            tree = t;
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.roots() {
            s.insert(*r.data());
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }
    }

    #[test]
    fn add_children() {
        let mut tree = Tree::empty();
        let mut cs = std::collections::HashSet::new();
        for i in 0..128 {
            let (n, _t) = tree.add_root_node(i);
            let ch1 = rand();
            let ch2 = rand();
            cs.insert((i, ch1));
            cs.insert((i, ch2));
            let chr1 = n.add_child(ch1);
            let chr2 = chr1.node.add_child(ch2);
            tree = chr2.tree;
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.roots() {
            s.insert(*r.data());
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }

        for r in tree.roots() {
            for ch in r.iter_children() {
                assert!(cs.contains(&(*r.data(), *ch.data())));
            }
        }
    }
}
