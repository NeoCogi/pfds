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
        let len = path.len();
        for i in 0..len {
            let parent = &path[len - i - 1];
            let mut children = parent.0.children.insert(new_path[i].clone());
            children = if i != 0 {
                children.remove(path[len - i].clone()) // remove the old node (old parent) after inserting the new modified one
            } else {
                children
            };

            let new_parent = Node(Arc::new(NodePriv {
                data: parent.0.data.clone(),
                children,
            }));
            new_path.push(new_parent.clone());
        }

        new_path.reverse();
        let root = new_path[0].clone();
        (new_path, Arc::new(Self { root }))
    }

    pub fn remove_node(&self, path: &Vec<Node<D>>) -> (Vec<Node<D>>, Arc<Self>) {
        assert!(path.len() >= 2);
        let mut new_path: Vec<Node<D>> = Vec::new();
        let len = path.len();
        for i in 0..len - 1 {
            let parent = &path[len - i - 2];
            let mut children = parent.0.children.remove(path[len - i - 1].clone());
            children = if i != 0 {
                children.insert(new_path[i - 1].clone()) // insert the modified node (old parent rebuilt) after removing the old one
            } else {
                children
            };

            let new_parent = Node(Arc::new(NodePriv {
                data: parent.0.data.clone(),
                children,
            }));
            new_path.push(new_parent.clone());
        }

        new_path.reverse();
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

    fn flatten_priv(node: &Node<D>, res: &mut Vec<Node<D>>) {
        for c in node.iter_children() {
            res.push(c.clone());
            Self::flatten_priv(&c, res);
        }
    }

    pub fn flatten(&self) -> Vec<Node<D>> {
        let mut res = vec![self.tree.root.clone()];
        Self::flatten_priv(&self.tree.root, &mut res);
        res
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Sub;
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
            assert_eq!(r.iter_children().count(), 2);
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

    #[test]
    fn remove_roots() {
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

        let mut r = std::collections::HashSet::new();

        while let Some(n) = tree.root().iter_children().next() {
            r.insert(*n.data());
            let (p, t) = tree.remove_node(&vec![tree.root(), n]);
            tree = t;
            for c in tree.root().iter_children() {
                assert!(!r.contains(c.data()))
            }

            let isub = s.sub(&r);
            for c in tree.root().iter_children() {
                assert!(isub.contains(c.data()))
            }
        }
    }

    #[test]
    fn remove_roots_and_nodes() {
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
            assert_eq!(r.iter_children().count(), 2);
            let d = *r.data();
            s.insert(d);
            for cc in r.iter_children() {
                s.insert(*cc.data());
            }
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }

        let mut r = std::collections::HashSet::new();

        //
        // highly unadvisable ("reassign tree") inside the loop, since tree and children order will change
        //
        let mut checked_root: std::collections::HashSet<i32> = std::collections::HashSet::new();
        let mut iter = tree.root().iter_children();
        let mut i = 0;
        while let Some(n) = iter.next() {
            if i >= 128 {
                break;
            }

            let d = *n.data();
            if checked_root.contains(&d) {
                continue;
            }

            i += 1;
            checked_root.insert(*n.data());
            assert_eq!(n.iter_children().count(), 2);
            let child = n.iter_children().last().unwrap();
            r.insert(*child.data());
            let (p, t) = tree.remove_node(&vec![tree.root(), n, child]);
            tree = t;
            for c in tree.root().iter_children() {
                assert!(!r.contains(c.data()));
                for cc in c.iter_children() {
                    assert!(!r.contains(cc.data()));
                }
            }

            let isub = s.sub(&r);
            for c in tree.root().iter_children() {
                assert!(isub.contains(c.data()));
                for cc in c.iter_children() {
                    assert!(isub.contains(cc.data()))
                }
            }
            iter = tree.root().iter_children();
        }
    }
}
