use crate::{HashMap, HashSet, Hashable};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[derive(Clone)]
struct Node<D: Clone + Default>(Arc<NodePriv<D>>);

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
struct PathPriv<D: Clone + Default> {
    path: Vec<Node<D>>,
}

impl<D: Clone + Default> PathPriv<D> {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            path: vec![Node(Arc::new(NodePriv {
                data: D::default(),
                children: HashSet::empty(),
            }))],
        })
    }

    pub fn add_node(&self, data: D) -> Arc<Self> {
        assert!(self.path.len() >= 1);
        for i in 1..self.path.len() {
            assert!(self.path[i - 1].0.children.exist(self.path[i].clone()));
        }

        let new_child = Node::new_with_data(data);
        let mut new_path = vec![new_child.clone()];
        let len = self.path.len();
        for i in 0..len {
            let parent = &self.path[len - i - 1];
            let mut children = parent.0.children.insert(new_path[i].clone());
            children = if i != 0 {
                children.remove(self.path[len - i].clone()) // remove the old node (old parent) after inserting the new modified one
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

        Arc::new(Self { path: new_path })
    }

    pub fn remove_node(&self) -> Arc<Self> {
        assert!(self.path.len() >= 2);
        let mut new_path: Vec<Node<D>> = Vec::new();
        let len = self.path.len();
        for i in 0..len - 1 {
            let parent = &self.path[len - i - 2];
            let mut children = parent.0.children.remove(self.path[len - i - 1].clone());
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
        Arc::new(Self { path: new_path })
    }
}

#[derive(Clone)]
pub struct Path<D: Clone + Default> {
    path: Arc<PathPriv<D>>,
}

impl<D: Clone + Default> Path<D> {
    pub fn empty() -> Self {
        Self { path: PathPriv::empty() }
    }

    pub fn add_node(&self, data: D) -> Self {
        Self {
            path: self.path.add_node(data),
        }
    }

    pub fn remove_node(&self) -> Self {
        Self { path: self.path.remove_node() }
    }

    pub fn root(&self) -> Self {
        Self {
            path: Arc::new(PathPriv {
                path: vec![self.path.path[0].clone()],
            }),
        }
    }

    pub fn data(&self) -> &D {
        self.path.path.last().unwrap().data()
    }

    pub fn children(&self) -> Vec<Self> {
        let mut res = Vec::new();
        for c in self.path.path.last().unwrap().iter_children() {
            let mut new_path = self.path.path.clone();
            new_path.push(c);

            res.push(Self {
                path: Arc::new(PathPriv { path: new_path }),
            });
        }
        res
    }

    pub fn parent(&self) -> Self {
        let len = self.path.path.len();
        let parent_path = Vec::from(&self.path.path[0..len - 1]);
        Self {
            path: Arc::new(PathPriv { path: parent_path }),
        }
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
        let mut tree = Path::empty();
        for i in 0..128 {
            let t = tree.add_node(i);
            tree = t.parent();
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.children() {
            s.insert(*r.data());
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }
    }

    #[test]
    fn add_children() {
        let mut tree = Path::empty();
        let mut cs = std::collections::HashSet::new();
        for i in 0..128 {
            let node = tree.add_node(i);
            let ch1 = rand();
            let ch2 = rand();
            cs.insert((i, ch1));
            cs.insert((i, ch2));
            let node1 = node.add_node(ch1);
            let node2 = node1.parent().add_node(ch2);
            tree = node2.root();
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.children() {
            let d = *r.data();
            s.insert(d);
            assert_eq!(r.children().len(), 2);
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }

        for r in tree.children() {
            for ch in r.children() {
                assert!(cs.contains(&(*r.data(), *ch.data())));
            }
        }
    }

    #[test]
    fn remove_roots() {
        let mut tree = Path::empty();
        for i in 0..128 {
            let t = tree.add_node(i);
            tree = t.root();
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.root().children() {
            s.insert(*r.data());
        }

        for i in 0..128 {
            assert!(s.contains(&i));
        }

        let mut r = std::collections::HashSet::new();

        while let Some(n) = tree.root().children().iter().next() {
            r.insert(*n.data());
            let t = n.remove_node();
            tree = t;
            for c in tree.root().children() {
                assert!(!r.contains(c.data()))
            }

            let isub = s.sub(&r);
            for c in tree.root().children() {
                assert!(isub.contains(c.data()))
            }
        }
    }

    #[test]
    fn remove_roots_and_nodes() {
        let mut tree = Path::empty();
        let mut cs = std::collections::HashSet::new();
        for i in 0..128 {
            let node = tree.add_node(i);
            let ch1 = rand();
            let ch2 = rand();
            cs.insert((i, ch1));
            cs.insert((i, ch2));
            let node1 = node.add_node(ch1);
            let node2 = node1.parent().add_node(ch2);
            tree = node2.root();
        }

        let mut s = std::collections::HashSet::new();
        for r in tree.root().children() {
            assert_eq!(r.children().len(), 2);
            let d = *r.data();
            s.insert(d);
            for cc in r.children() {
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
        let mut i = 0;
        let mut children = tree.root().children();
        let mut iter = children.iter();
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
            assert_eq!(n.children().len(), 2);
            let children1 = n.children();
            let child = children1.last().unwrap();
            r.insert(*child.data());
            let t = child.remove_node();
            tree = t.root();
            for c in tree.root().children() {
                assert!(!r.contains(c.data()));
                for cc in c.children() {
                    assert!(!r.contains(cc.data()));
                }
            }

            let isub = s.sub(&r);
            for c in tree.root().children() {
                assert!(isub.contains(c.data()));
                for cc in c.children() {
                    assert!(isub.contains(cc.data()))
                }
            }
            children = tree.root().children();
            iter = children.iter();
        }
    }
}
