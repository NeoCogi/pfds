use crate::{HashMap, HashSet, Hashable};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[derive(Clone)]
pub struct Node<D: Clone + Default>(Arc<NodePriv<D>>);

#[derive(Clone)]
struct NodePriv<D: Clone + Default> {
    data: D,
    children: HashSet<Node<D>>,
}

impl<D: Clone + Default> Hashable for Node<D> {
    fn hash(&self) -> u64 {
        Arc::as_ptr(&self.0) as usize as u64
    }
}

impl<D: Clone + Default> PartialEq for Node<D> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<D: Clone + Default> Eq for Node<D> {}

impl<D: Clone + Default> Node<D> {
    pub fn data(&self) -> &D {
        &self.0.data
    }

    pub fn iter_children<'a>(&self) -> crate::hashset::Iter<'a, Self> {
        self.0.children.iter()
    }

    fn new_with_data(data: D) -> Self {
        Self(Arc::new(NodePriv {
            data,
            children: HashSet::empty(),
        }))
    }
}

#[derive(Clone)]
struct TreeIntern<D: Clone + Default> {
    root: Node<D>,
}

impl<D: Clone + Default> TreeIntern<D> {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            root: Node(Arc::new(NodePriv {
                data: D::default(),
                children: HashSet::empty(),
            })),
        })
    }

    pub fn add_node(&self, path: &Vec<Node<D>>, data: D) -> (Vec<Node<D>> /* node */, Arc<Self>) {
        assert!(path.len() >= 1);
        for i in 1..path.len() {
            assert!(path[i - 1].0.children.exist(path[i].clone()));
        }

        let new_child = Node::new_with_data(data);
        let mut new_path = vec![new_child.clone()];
        let mut old_node = None;
        let mut current_node = new_child.clone();
        let len = path.len();
        for i in 0..len {
            let parent = &path[len - i - 1];
            let mut children = parent.0.children.insert(current_node.clone());
            children = match old_node {
                Some(p) => children.remove(p), // remove the old node (old parent) after inserting the new modified one
                None => children,
            };

            let new_parent = Node(Arc::new(NodePriv {
                data: parent.0.data.clone(),
                children,
            }));
            new_path.push(new_parent.clone());
            current_node = new_parent;
            old_node = Some(parent.clone());
        }

        new_path.reverse();
        let root = new_path[0].clone();
        (new_path, Arc::new(Self { root }))
    }

    pub fn remove_node(&self, path: &Vec<Node<D>>) -> (Vec<Node<D>>, Arc<Self>) {
        assert!(path.len() >= 2);
        let mut new_path = Vec::new();
        let mut old_node = None;
        let len = path.len();
        let mut current_node = path[len - 1].clone();
        for i in 0..len - 1 {
            let parent = &path[len - i - 2];
            let mut children = parent.0.children.remove(current_node.clone());
            children = match old_node {
                Some(p) => children.insert(p), // insert the modified node (old parent rebuilt) after removing the old one
                None => children,
            };

            let new_parent = Node(Arc::new(NodePriv {
                data: parent.0.data.clone(),
                children,
            }));
            new_path.push(new_parent.clone());
            old_node = Some(new_parent);
        }

        let root = new_path[0].clone();
        (new_path, Arc::new(Self { root }))
    }
}

#[derive(Clone)]
pub struct Tree<D: Clone + Default> {
    tree: Arc<TreeIntern<D>>,
}

impl<D: Clone + Default> Tree<D> {
    pub fn empty() -> Self {
        Self { tree: TreeIntern::empty() }
    }

    pub fn add_node(&self, path: &Vec<Node<D>>, data: D) -> (Vec<Node<D>>, Self) {
        let (new_path, tree) = self.tree.add_node(path, data);
        (new_path, Self { tree })
    }

    pub fn remove_node(&self, path_to_node: &Vec<Node<D>>) -> (Vec<Node<D>>, Self) {
        let (new_path, tree) = self.tree.remove_node(path_to_node);
        (new_path, Self { tree })
    }

    pub fn root(&self) -> Node<D> {
        self.tree.root.clone()
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
            let (_, t) = tree.add_node(&vec![tree.root()], i);
            tree = t;
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.root().iter_children() {
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
            let (p, tree_) = tree.add_node(&vec![tree.root()], i);
            let ch1 = rand();
            let ch2 = rand();
            cs.insert((i, ch1));
            cs.insert((i, ch2));
            let (p, tree_) = tree_.add_node(&vec![p[0].clone(), p[1].clone()], ch1);
            let (_, tree_) = tree_.add_node(&vec![p[0].clone(), p[1].clone()], ch2);
            tree = tree_;
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.root().iter_children() {
            let d = *r.data();
            s.insert(d);
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }

        for r in tree.root().iter_children() {
            for ch in r.iter_children() {
                assert!(cs.contains(&(*r.data(), *ch.data())));
            }
        }
    }
}
