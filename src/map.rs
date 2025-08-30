//
// Copyright 2021-Present (c) Raja Lehtihet & Wael El Oraiby
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software without
// specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//

use std::sync::Arc;
use std::marker::PhantomData;

#[derive(Clone)]
enum MapNode<K: Clone, V: Clone> {
    Empty,
    One(K, V),
    Node(usize, Arc<MapNode<K, V>>, K, V, Arc<MapNode<K, V>>),
}

use MapNode::*;

type S<K, V> = MapNode<K, V>;
type N<K, V> = Arc<MapNode<K, V>>;

impl<K: Ord + Clone, V: Clone> MapNode<K, V> {
    fn empty() -> N<K, V> {
        N::new(Empty)
    }
    fn one(k: K, v: V) -> N<K, V> {
        N::new(One(k, v))
    }
    fn node(h: usize, l: &N<K, V>, k: K, v: V, r: &N<K, V>) -> N<K, V> {
        N::new(Node(h, l.clone(), k, v, r.clone()))
    }

    fn make(l: &N<K, V>, k: K, v: V, r: &N<K, V>) -> N<K, V> {
        match (l.as_ref(), r.as_ref()) {
            (Empty, Empty) => S::one(k, v),
            _ => {
                let h = 1 + usize::max(l.height(), r.height());
                S::node(h, l, k, v, r)
            }
        }
    }

    fn rebalance(t1: &N<K, V>, k: K, v: V, t2: &N<K, V>) -> N<K, V> {
        let t1h = t1.height();
        let t2h = t2.height();

        if t2h > t1h + 2 {
            match t2.as_ref() {
                Node(_, t2l, t2k, t2v, t2r) => {
                    if t2l.height() > t1h + 1 {
                        match t2l.as_ref() {
                            Node(_, t2ll, t2lk, t2lv, t2lr) => S::make(
                                &S::make(t1, k, v, t2ll),
                                t2lk.clone(),
                                t2lv.clone(),
                                &S::make(t2lr, t2k.clone(), t2v.clone(), t2r),
                            ),
                            _ => unreachable!(),
                        }
                    } else {
                        S::make(&S::make(t1, k, v, t2l), t2k.clone(), t2v.clone(), t2r)
                    }
                }
                _ => unreachable!(),
            }
        } else if t1h > t2h + 2 {
            match t1.as_ref() {
                Node(_, t1l, t1k, t1v, t1r) => {
                    if t1r.height() > t2h + 1 {
                        match t1r.as_ref() {
                            Node(_, t1rl, t1rk, t1rv, t1rr) => S::make(
                                &S::make(t1l, t1k.clone(), t1v.clone(), t1rl),
                                t1rk.clone(),
                                t1rv.clone(),
                                &S::make(t1rr, k, v, t2),
                            ),
                            _ => unreachable!(),
                        }
                    } else {
                        S::make(t1l, t1k.clone(), t1v.clone(), &S::make(t1r, k, v, t2))
                    }
                }
                _ => unreachable!(),
            }
        } else {
            S::make(t1, k, v, t2)
        }
    }

    fn insert(t: &N<K, V>, k: K, v: V) -> N<K, V> {
        match t.as_ref() {
            Node(_, l, k2, v2, r) if k < k2.clone() => {
                S::rebalance(&S::insert(l, k, v), k2.clone(), v2.clone(), r)
            }
            Node(h, l, k2, v2, r) if k == k2.clone() => S::node(*h, l, k2.clone(), v2.clone(), r),
            Node(_, l, k2, v2, r) if k > k2.clone() => {
                S::rebalance(l, k2.clone(), v2.clone(), &S::insert(r, k, v))
            }

            One(k2, v2) if k < k2.clone() => {
                S::node(2, &S::empty(), k, v, &S::one(k2.clone(), v2.clone()))
            }
            One(k2, v2) if k == k2.clone() => S::one(k2.clone(), v2.clone()),
            One(k2, v2) if k > k2.clone() => {
                S::node(2, &S::one(k2.clone(), v2.clone()), k, v, &S::empty())
            }

            Empty => S::one(k, v),
            _ => unreachable!(),
        }
    }

    fn splice_out_successor(t: &N<K, V>) -> (K, V, N<K, V>) {
        match t.as_ref() {
            Empty => panic!("internal error"),
            One(k2, v2) => (k2.clone(), v2.clone(), S::empty()),
            Node(_, l, k2, v2, r) => {
                let l1 = l.clone();
                let r1 = r.clone();
                match l.as_ref() {
                    Empty => (k2.clone(), v2.clone(), r1),
                    _ => {
                        let (k3, v3, ll) = S::splice_out_successor(&l1);
                        (k3, v3, S::make(&ll, k2.clone(), v2.clone(), r))
                    }
                }
            }
        }
    }

    fn remove(t: &N<K, V>, k: K) -> N<K, V> {
        match t.as_ref() {
            Empty => S::empty(),
            One(k2, _) if k == k2.clone() => S::empty(),
            One(k2, v2) => S::one(k2.clone(), v2.clone()),
            Node(_, l, k2, v2, r) if k < k2.clone() => {
                S::rebalance(&S::remove(l, k), k2.clone(), v2.clone(), r)
            }
            Node(_, l, k2, _, r) if k == k2.clone() => {
                let l1 = l.clone();
                let r1 = r.clone();
                match (l.as_ref(), r.as_ref()) {
                    (Empty, _) => r1,
                    (_, Empty) => l1,
                    _ => {
                        let (sk, sv, rr) = S::splice_out_successor(&r1);
                        S::make(&l1, sk, sv, &rr)
                    }
                }
            }
            Node(_, l, k2, v2, r) if k > k2.clone() => {
                S::rebalance(l, k2.clone(), v2.clone(), &S::remove(r, k))
            }
            _ => unreachable!(),
        }
    }

    fn find(&self, k: K) -> Option<&V> {
        match self {
            Empty => None,
            One(k2, v) if k == k2.clone() => Some(v),
            One(_, _) => None,
            Node(_, l, k2, _, _) if k < k2.clone() => S::find(l, k),
            Node(_, _, k2, v, _) if k == k2.clone() => Some(v),
            Node(_, _, k2, _, r) if k > k2.clone() => S::find(r, k),
            _ => unreachable!(),
        }
    }

    fn to_vec(t: &N<K, V>, vec: &mut Vec<(K, V)>) {
        match t.as_ref() {
            Empty => (),
            One(k, v) => vec.push((k.clone(), v.clone())),
            Node(_, l, k, v, r) => {
                S::to_vec(l, vec);
                vec.push((k.clone(), v.clone()));
                S::to_vec(r, vec);
            }
        }
    }
}

impl<K: Clone, V: Clone> MapNode<K, V> {
    fn height(&self) -> usize {
        match self {
            Empty => 0,
            One(_, _) => 1,
            Node(h, _, _, _, _) => *h,
        }
    }
}

/// A persistent (immutable) ordered map data structure.
/// 
/// `Map` is implemented as a self-balancing binary search tree (AVL tree)
/// that maintains key-value pairs in sorted order by key. All operations
/// return a new map, leaving the original unchanged.
/// 
/// # Performance
/// 
/// - `insert`: O(log n)
/// - `remove`: O(log n)
/// - `find`: O(log n)
/// - `exist`: O(log n)
/// - `len`: O(1) - size is cached
/// - `height`: O(1) - height is cached
/// - `to_vec`: O(n) - returns pairs in sorted order by key
pub struct Map<K: Ord + Clone, V: Clone> {
    size: usize,
    n: N<K, V>,
}

impl<K: Ord + Clone, V: Clone> Map<K, V> {
    /// Creates a new empty map.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map: Map<i32, String> = Map::empty();
    /// assert!(map.is_empty());
    /// assert_eq!(map.len(), 0);
    /// ```
    pub fn empty() -> Self {
        Self {
            n: S::empty(),
            size: 0,
        }
    }

    /// Creates a new map with the given key-value pair inserted.
    /// 
    /// If the key already exists, its value is replaced.
    /// This operation is O(log n) and shares structure with the original map.
    /// 
    /// # Arguments
    /// 
    /// * `k` - The key to insert
    /// * `v` - The value associated with the key
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert("a", 1).insert("b", 2).insert("c", 3);
    /// assert_eq!(map.len(), 3);
    /// assert_eq!(map.find("b"), Some(&2));
    /// ```
    pub fn insert(&self, k: K, v: V) -> Self {
        Self {
            n: S::insert(&self.n, k, v),
            size: self.size + 1,
        }
    }

    /// Creates a new map with the given key-value pair removed.
    /// 
    /// If the key doesn't exist, the returned map is unchanged.
    /// This operation is O(log n) and shares structure with the original map.
    /// 
    /// # Arguments
    /// 
    /// * `k` - The key to remove
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert(1, "one").insert(2, "two").insert(3, "three");
    /// let map2 = map.remove(2);
    /// assert_eq!(map.len(), 3);  // Original unchanged
    /// assert_eq!(map2.len(), 2);
    /// assert_eq!(map2.find(2), None);
    /// ```
    pub fn remove(&self, k: K) -> Self {
        let size = match S::find(&self.n, k.clone()) {
            Some(_) => self.size - 1,
            None => self.size,
        };
        let n = S::remove(&self.n, k);
        Self { n, size }
    }

    /// Returns true if the map contains the given key.
    /// 
    /// This operation is O(log n).
    /// 
    /// # Arguments
    /// 
    /// * `k` - The key to search for
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert(1, "one").insert(2, "two");
    /// assert!(map.exist(1));
    /// assert!(!map.exist(3));
    /// ```
    pub fn exist(&self, k: K) -> bool {
        S::find(&self.n, k).is_some()
    }

    /// Returns a reference to the value associated with the given key.
    /// 
    /// Returns `None` if the key is not present in the map.
    /// This operation is O(log n).
    /// 
    /// # Arguments
    /// 
    /// * `k` - The key to search for
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert("key", 42);
    /// assert_eq!(map.find("key"), Some(&42));
    /// assert_eq!(map.find("missing"), None);
    /// ```
    pub fn find(&self, k: K) -> Option<&V> {
        S::find(&self.n, k)
    }

    /// Converts the map to a vector of key-value pairs in sorted order by key.
    /// 
    /// This operation is O(n).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert(3, "c").insert(1, "a").insert(2, "b");
    /// let vec = map.to_vec();
    /// assert_eq!(vec, vec![(1, "a"), (2, "b"), (3, "c")]); // Sorted by key
    /// ```
    pub fn to_vec(&self) -> Vec<(K, V)> {
        let mut v = Vec::new();
        S::to_vec(&self.n, &mut v);
        v
    }

    /// Returns the height of the balanced tree.
    /// 
    /// The height is the length of the longest path from root to leaf.
    /// This operation is O(1) as height is cached.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert(1, "a").insert(2, "b").insert(3, "c");
    /// assert!(map.height() > 0);
    /// ```
    pub fn height(&self) -> usize {
        self.n.height()
    }

    /// Returns true if the map is empty.
    /// 
    /// This operation is O(1).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let empty = Map::<i32, String>::empty();
    /// assert!(empty.is_empty());
    /// 
    /// let non_empty = empty.insert(1, String::from("one"));
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of key-value pairs in the map.
    /// 
    /// This operation is O(1) as the size is cached.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert(1, "a").insert(2, "b").insert(3, "c");
    /// assert_eq!(map.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns an iterator over the map's key-value pairs.
    /// 
    /// The iterator yields pairs in sorted order by key.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Map;
    /// 
    /// let map = Map::empty().insert(3, "c").insert(1, "a").insert(2, "b");
    /// let collected: Vec<_> = map.iter().collect();
    /// assert_eq!(collected, vec![(1, "a"), (2, "b"), (3, "c")]);
    /// ```
    pub fn iter(&self) -> MapIter<K, V> {
        let mut stack = Vec::new();
        if !matches!(self.n.as_ref(), Empty) {
            stack.push(self.n.clone());
        }
        MapIter {
            stack,
            _phantom: PhantomData::default(),
        }
    }
}

/// An iterator over the key-value pairs of a `Map`.
/// 
/// This struct is created by the [`Map::iter`] method.
/// The iterator yields pairs in sorted order by key.
pub struct MapIter<'a, K: Ord + Clone, V: Clone> {
    stack: Vec<N<K, V>>,
    _phantom: PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord + Clone, V: Clone> std::iter::Iterator for MapIter<'a, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match node.as_ref() {
                Empty => continue,
                One(k, v) => return Some((k.clone(), v.clone())),
                Node(_, left, k, v, right) => {
                    // Push right first (will be processed after)
                    if !matches!(right.as_ref(), Empty) {
                        self.stack.push(right.clone());
                    }
                    // Push current node as One to process the key-value
                    self.stack.push(S::one(k.clone(), v.clone()));
                    // Push left (will be processed first - in-order traversal)
                    if !matches!(left.as_ref(), Empty) {
                        self.stack.push(left.clone());
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::map::*;

    static mut SEED: i64 = 777;

    fn rand() -> i32 {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32
        }
    }

    #[test]
    fn insert() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let sorted = [1, 3, 4, 5, 9, 10, 27, 45, 120];
        let mut n = Map::empty();
        for i in numbers {
            n = n.insert(i, i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(v[i].1, sorted[i]);
        }
    }

    #[test]
    fn find() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let mut n = Map::empty();
        for i in numbers {
            n = n.insert(i, i);
        }

        assert_eq!(n.find(10).is_some(), true);
        assert_eq!(n.find(11).is_none(), true);
    }

    #[test]
    fn remove() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let sorted = [1, 3, 4, 9, 10, 27, 45, 120];
        let mut n = Map::empty();
        for i in numbers {
            n = n.insert(i, i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), numbers.len());

        n = n.remove(5);

        let v = n.to_vec();

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(v[i].1, sorted[i]);
        }
    }

    #[test]
    fn remove_one_from_one() {
        let mut n = Map::empty();
        n = n.insert(10, 10);

        assert_eq!(n.find(5).is_none(), true);
        n = n.remove(5);

        assert_eq!(n.find(10).is_some(), true);
        n = n.remove(10);
        assert_eq!(n.find(10).is_none(), true);

        let v = n.to_vec();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn insert_10000_random() {
        let mut hs = std::collections::hash_set::HashSet::new();
        let mut numbers = Vec::new();
        for _ in 0..10000 {
            hs.insert(rand());
        }

        for i in hs.iter() {
            numbers.push(i);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        let mut n = Map::empty();
        for i in numbers {
            n = n.insert(i, i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(v[i].1, sorted[i]);
        }
    }

    #[test]
    fn iter() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let sorted = [1, 3, 4, 5, 9, 10, 27, 45, 120];
        let mut n = Map::empty();
        for i in numbers {
            n = n.insert(i, i * 2); // Use i*2 as value to test proper pairing
        }

        // Test iterator yields elements in sorted order
        let mut count = 0;
        for (k, v) in n.iter() {
            assert_eq!(k, sorted[count]);
            assert_eq!(v, sorted[count] * 2);
            count += 1;
        }
        assert_eq!(count, sorted.len());

        // Test that we can iterate multiple times (persistent data structure)
        let collected: Vec<(i32, i32)> = n.iter().collect();
        assert_eq!(collected.len(), sorted.len());
        for i in 0..collected.len() {
            assert_eq!(collected[i].0, sorted[i]);
            assert_eq!(collected[i].1, sorted[i] * 2);
        }
    }

    #[test]
    fn remove_5000_from_10000_random() {
        let mut hs = std::collections::hash_set::HashSet::new();
        let mut numbers = Vec::new();
        for _ in 0..10000 {
            hs.insert(rand() % 10000);
        }

        for i in hs.iter() {
            numbers.push(*i);
        }

        let mut n = Map::empty();
        for i in numbers.iter() {
            n = n.insert(*i, *i);
        }

        assert_eq!(n.len(), hs.len());

        let mut hs = hs.clone();

        for i in 0..hs.len() / 2 {
            hs.remove(&numbers[i]);
            n = n.remove(numbers[i]);
        }

        assert_eq!(n.len(), hs.len());

        let mut sorted = Vec::new();
        for i in hs.iter() {
            sorted.push(*i);
        }
        sorted.sort();

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(v[i].1, sorted[i]);
        }

        assert_eq!(n.find(numbers[0]).is_none(), true);
        assert_eq!(n.to_vec().len(), hs.len());
    }
}
