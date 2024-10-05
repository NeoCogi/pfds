use crate::{HashSet, Hashable};
use std::{ops::Deref, sync::Arc};

pub trait TreeAcc<D: Clone> {
    fn push(&mut self, data: &D);
    fn pop(&mut self);
}

#[derive(Clone)]
struct Node<D: Clone>(Arc<NodePriv<D>>);

#[derive(Clone)]
struct NodePriv<D: Clone> {
    data: D,
    children: HashSet<Node<D>>,
}

impl<D: Clone> Hashable for Node<D> {
    fn hash(&self) -> u64 {
        Arc::as_ptr(&self.0) as usize as u64
    }
}

impl<D: Clone> PartialEq for Node<D> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<D: Clone> Eq for Node<D> {}

impl<D: Clone> Node<D> {
    pub fn data(&self) -> &D {
        &self.0.data
    }

    pub fn iter_children<'a>(&self) -> crate::hashset::HSIter<'a, Self> {
        self.0.children.iter()
    }

    fn new(data: D, children: HashSet<Node<D>>) -> Self {
        Self(Arc::new(NodePriv { data, children }))
    }

    fn apply<F: FnOnce(&D) -> Option<D>>(&self, f: F) -> Option<Self> {
        let new_data = f(self.data());
        new_data.map(|data| {
            Self(Arc::new(NodePriv {
                data,
                children: self.0.children.clone(),
            }))
        })
    }

    fn apply_recursive<F: FnMut(&D) -> Option<D>>(&self, f: &mut F) -> Option<Self> {
        let mut changed = false;
        let mut children = HashSet::empty();

        // TODO: using while let Some(c) = self.0.children.iter() seems to make this hangs: Investigate!!!!
        for c in self.0.children.iter() {
            let child = match c.apply_recursive(f) {
                Some(c) => {
                    changed |= true;
                    c
                }
                None => c,
            };
            children = children.insert(child);
        }

        let children = if changed {
            children
        } else {
            self.0.children.clone()
        };

        let new_data = f(self.data());
        changed |= new_data.is_some();
        let data = match new_data {
            Some(data) => data,
            None => self.0.data.clone(),
        };

        if changed {
            Some(Self(Arc::new(NodePriv { data, children })))
        } else {
            None
        }
    }

    fn apply_acc_recursive<Acc: TreeAcc<D>, F: FnMut(&mut Acc, &D) -> Option<D>>(
        &self,
        acc: &mut Acc,
        f: &mut F,
    ) -> Option<Self> {
        let mut changed = false;
        let mut children = HashSet::empty();

        acc.push(self.data());
        let new_data = f(acc, self.data());

        // TODO: using while let Some(c) = self.0.children.iter() seems to make this hangs: Investigate!!!!
        for c in self.0.children.iter() {
            let child = match c.apply_acc_recursive(acc, f) {
                Some(c) => {
                    changed |= true;
                    c
                }
                None => c,
            };
            children = children.insert(child);
        }

        let children = if changed {
            children
        } else {
            self.0.children.clone()
        };
        acc.pop();

        changed |= new_data.is_some();
        let data = match new_data {
            Some(data) => data,
            None => self.0.data.clone(),
        };

        if changed {
            Some(Self(Arc::new(NodePriv { data, children })))
        } else {
            None
        }
    }
    ///
    /// Returns a new tree containing only the nodes for which the given predicate "f" returns "true"
    ///
    fn filter_recursive<F: Fn(&D) -> bool>(&self, f: Arc<F>) -> Option<Self> {
        match f(self.data()) {
            true => {
                let mut children = HashSet::empty();
                for c in self.iter_children() {
                    match c.filter_recursive(f.clone()) {
                        Some(cc) => children = children.insert(cc),
                        None => (),
                    }
                }
                Some(Self(Arc::new(NodePriv {
                    data: self.data().clone(),
                    children,
                })))
            }
            false => None,
        }
    }

    pub fn map_data<F: FnMut(&D) -> Option<D>>(&self, f: &mut F) -> Option<Self> {
        let mut children: HashSet<Node<D>> = HashSet::empty();
        let mut children_changed = false;
        for c in self.0.children.iter() {
            children = match c.map_data(f) {
                Some(d) => {
                    children_changed = true;
                    children.insert(d)
                }
                None => children.insert(c.clone()),
            };
        }

        match f(self.data()) {
            Some(d) => Some(if children_changed {
                Node::new(d, children)
            } else {
                Node::new(d, self.0.children.clone())
            }),
            None => {
                if children_changed {
                    Some(Node::new(self.data().clone(), children))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone)]
struct PathPriv<D: Clone> {
    node_vec: Vec<Node<D>>,
}

impl<D: Clone> PathPriv<D> {
    pub fn new(data: D) -> Arc<Self> {
        Arc::new(Self {
            node_vec: vec![Node(Arc::new(NodePriv {
                data,
                children: HashSet::empty(),
            }))],
        })
    }

    pub fn add_node(&self, data: D) -> Arc<Self> {
        assert!(self.node_vec.len() >= 1);
        for i in 1..self.node_vec.len() {
            assert!(self.node_vec[i - 1]
                .0
                .children
                .exist(self.node_vec[i].clone()));
        }

        let new_child = Node::new(data, HashSet::empty());
        let mut new_path = vec![new_child.clone()];
        let len = self.node_vec.len();
        for i in 0..len {
            let parent = &self.node_vec[len - i - 1];
            let mut children = parent.0.children.insert(new_path[i].clone());
            children = if i != 0 {
                children.remove(self.node_vec[len - i].clone()) // remove the old node (old parent) after inserting the new modified one
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

        Arc::new(Self { node_vec: new_path })
    }

    pub fn remove_node(&self) -> Arc<Self> {
        assert!(self.node_vec.len() > 1);

        let mut new_path: Vec<Node<D>> = Vec::new();
        let len = self.node_vec.len();
        for i in 0..len - 1 {
            let parent = &self.node_vec[len - i - 2];
            let mut children = parent.0.children.remove(self.node_vec[len - i - 1].clone());
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
        Arc::new(Self { node_vec: new_path })
    }

    pub fn set_data(&self, data: D) -> Arc<Self> {
        let rm = self.remove_node();
        rm.add_node(data)
    }

    fn node(&self) -> Node<D> {
        self.node_vec.last().unwrap().clone()
    }

    fn propagate_last_node_change(&self, node: Node<D>) -> Arc<Self> {
        let new_child = node;
        let mut new_path = vec![new_child.clone()];
        let len = self.node_vec.len();
        for i in 0..len - 1 {
            let parent = &self.node_vec[len - i - 2];

            let children = parent
                .0
                .children
                // remove the old child node;
                .remove(self.node_vec[len - i - 1].clone())
                // insert the new child node
                .insert(new_path[i].clone());

            assert!(parent.0.children.len() == children.len());

            let new_parent = Node(Arc::new(NodePriv {
                data: parent.0.data.clone(),
                children,
            }));
            new_path.push(new_parent.clone());
        }
        new_path.reverse();
        Arc::new(Self { node_vec: new_path })
    }

    fn apply<F: FnOnce(&D) -> Option<D>>(&self, f: F) -> Option<Arc<Self>> {
        self.node()
            .apply(f)
            .map(|n| self.propagate_last_node_change(n))
    }

    fn apply_recursive<F: FnMut(&D) -> Option<D>>(&self, f: &mut F) -> Option<Arc<Self>> {
        self.node()
            .apply_recursive(f)
            .map(|n| self.propagate_last_node_change(n))
    }

    fn apply_acc_recursive<Acc: TreeAcc<D>, F: FnMut(&mut Acc, &D) -> Option<D>>(
        &self,
        init: &mut Acc,
        f: &mut F,
    ) -> Option<Arc<Self>> {
        self.node()
            .apply_acc_recursive(init, f)
            .map(|n| self.propagate_last_node_change(n))
    }

    fn filter_recursive<F: Fn(&D) -> bool>(&self, f: F) -> Option<Arc<Self>> {
        self.node()
            .filter_recursive(Arc::new(f))
            .map(|n| self.propagate_last_node_change(n))
    }
}

#[derive(Clone)]
pub struct Path<D: Clone> {
    path: Arc<PathPriv<D>>,
}

impl<D: Clone> Path<D> {
    pub fn new(root_data: D) -> Self {
        Self {
            path: PathPriv::new(root_data),
        }
    }

    pub fn add_node(&self, data: D) -> Self {
        Self {
            path: self.path.add_node(data),
        }
    }

    pub fn remove_node(&self) -> Self {
        Self {
            path: self.path.remove_node(),
        }
    }

    pub fn root(&self) -> Self {
        Self {
            path: Arc::new(PathPriv {
                node_vec: vec![self.path.node_vec[0].clone()],
            }),
        }
    }

    pub fn data(&self) -> &D {
        self.path.node_vec.last().unwrap().data()
    }

    pub fn children(&self) -> Vec<Self> {
        let mut res = Vec::new();
        let iter = self.path.node_vec.last().unwrap().iter_children();
        for c in iter {
            let mut new_path = self.path.node_vec.clone();
            new_path.push(c);

            res.push(Self {
                path: Arc::new(PathPriv { node_vec: new_path }),
            });
        }
        res
    }

    pub fn parent(&self) -> Self {
        let len = self.path.node_vec.len();
        let parent_path = Vec::from(&self.path.node_vec[0..len - 1]);
        Self {
            path: Arc::new(PathPriv {
                node_vec: parent_path,
            }),
        }
    }

    pub fn set_data(&self, data: D) -> Self {
        Self {
            path: self.path.set_data(data),
        }
    }

    pub fn apply<F: FnOnce(&D) -> Option<D>>(&self, f: F) -> Self {
        match self.path.apply(f) {
            Some(path) => Self { path },
            None => self.clone(),
        }
    }

    pub fn apply_recursive<F: FnMut(&D) -> Option<D>>(&self, mut f: F) -> Self {
        match self.path.apply_recursive(&mut f) {
            Some(path) => Self { path },
            None => self.clone(),
        }
    }

    pub fn apply_acc_recursive<Acc: TreeAcc<D>, F: FnMut(&mut Acc, &D) -> Option<D>>(
        &self,
        init: &mut Acc,
        mut f: F,
    ) -> Self {
        match self.path.apply_acc_recursive(init, &mut f) {
            Some(path) => Self { path },
            None => self.clone(),
        }
    }

    fn flatten_recursive(node: &Self, res: &mut Vec<Self>) {
        res.push(node.clone());

        for c in node.children() {
            Self::flatten_recursive(&c, res);
        }
    }

    pub fn flatten(&self) -> Vec<Self> {
        let mut res = Vec::new();
        Self::flatten_recursive(&self, &mut res);
        res
    }

    pub fn len(&self) -> usize {
        self.path.node_vec.len()
    }

    pub fn filter_recursive<F: Fn(&D) -> bool>(&self, f: F) -> Option<Self> {
        match self.path.filter_recursive(f) {
            Some(path) => Some(Self { path }),
            None => None,
        }
    }

    // breath first
    pub fn iter_acc_recursive<Acc: TreeAcc<D>, F: FnMut(&mut Acc, &Path<D>)>(
        &self,
        init: &mut Acc,
        f: &mut F,
    ) {
        init.push(self.data());

        f(init, &self);
        for c in self.children().iter() {
            c.iter_acc_recursive(init, f);
        }

        init.pop();
    }

    // breath first
    #[inline(never)]
    pub fn iter_recursive<F: FnMut(&Path<D>)>(&self, f: &mut F) {
        f(self);
        for c in self.children().iter() {
            c.iter_recursive(f);
        }
    }

    pub fn remove_all_children(&self) -> Self {
        match self.path.node().0.children.len() {
            x if x > 0 => {
                let mut n = (*self.path.node().0).clone();
                n.children = HashSet::empty();
                let p = self.path.propagate_last_node_change(Node(Arc::new(n)));
                Self { path: p }
            }
            _ => self.clone(),
        }
    }

    pub fn map_data<F: FnMut(&D) -> Option<D>>(&self, mut f: F) -> Self {
        match self.path.node_vec[self.path.node_vec.len() - 1].map_data(&mut f) {
            Some(n) => Path {
                path: self.path.propagate_last_node_change(n),
            },
            None => self.clone(),
        }
    }
}

impl<D: Clone> PartialEq for Path<D> {
    fn eq(&self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.path, &other.path) {
            // check if the path length are different
            if self.path.node_vec.len() != other.path.node_vec.len() {
                return false;
            }
            // path is equal: check if node to node are equal
            for i in 0..self.path.node_vec.len() {
                if !Arc::ptr_eq(&self.path.node_vec[i].0, &other.path.node_vec[i].0) {
                    return false;
                }
            }

            println!("warning! slow path equality check");
            // they are equal
            true
        } else {
            true
        }
    }
}

impl<D: Clone> Eq for Path<D> {}

impl<D: Clone> Deref for Path<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T: Clone> TreeAcc<T> for Vec<T> {
    fn push(&mut self, data: &T) {
        self.push(data.clone());
    }

    fn pop(&mut self) {
        self.pop();
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::*;
    use std::ops::Sub;

    static mut SEED: i64 = 777;

    fn rand() -> i32 {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32
        }
    }

    #[test]
    fn add_roots() {
        let mut tree = Path::new(0);
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
        let mut tree = Path::new(0);
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
        let mut tree = Path::new(0);
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
        let mut tree = Path::new(0);
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

    #[test]
    fn apply_roots() {
        let mut tree = Path::new(0);
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

        for c in tree.children() {
            let new_c = c.apply(|x| Some(*x * 2));
            assert_eq!(*c.data() * 2, *new_c.data());
        }
    }

    #[test]
    fn apply_recursive_on_roots() {
        let mut tree = Path::new(0);
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

        for c in tree.children() {
            let new_c = c.apply_recursive(&mut |x: &i32| Some(*x * 2));
            assert_eq!(*c.data() * 2, *new_c.data());
        }
    }

    #[test]
    fn apply_recursive_children() {
        let mut tree = Path::new(0);
        let mut cs = std::collections::HashSet::new();
        for i in 0..128 {
            let node = tree.add_node(i);
            let ch1 = rand() & 0xFFFF;
            let ch2 = rand() & 0xFFFF;
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

        let mut n = 0;
        let tree_double = tree.apply_recursive(|x| {
            n = 3;
            Some(*x * 2)
        });
        assert_eq!(n, 3);

        for r in tree_double.children() {
            for ch in r.children() {
                assert!(cs.contains(&(*r.data() / 2, *ch.data() / 2)));
            }
        }
    }

    #[test]
    fn test_remove_all_children() {
        let mut tree = Path::new(0);
        for i in 1..10 {
            tree = tree.add_node(i)
        }

        let n9 = tree.parent();
        assert_eq!(*n9.data(), 8);

        let n8 = n9.parent();
        assert_eq!(*n8.data(), 7);

        assert_eq!(n8.children().len(), 1);
        assert_eq!(n8.remove_all_children().children().len(), 0);
    }

    #[test]
    fn test_map_data() {
        let mut tree = Path::new(0);
        for i in 1..10 {
            tree = tree.add_node(i)
        }

        let n9 = tree.parent();
        assert_eq!(*n9.data(), 8);

        let n8 = n9.parent();
        assert_eq!(*n8.data(), 7);

        assert_eq!(n8.children().len(), 1);

        let nn8 = n8.map_data(|_| None);
        assert!(nn8 == n8);
    }
}
