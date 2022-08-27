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

pub struct Map<K: Ord + Clone, V: Clone> {
    size: usize,
    n: N<K, V>,
}

impl<K: Ord + Clone, V: Clone> Map<K, V> {
    ///
    /// create and return a new empty map
    ///
    pub fn empty() -> Self {
        Self {
            n: S::empty(),
            size: 0,
        }
    }

    ///
    /// create and return a new map containing the new key, value pair
    ///
    pub fn insert(&self, k: K, v: V) -> Self {
        Self {
            n: S::insert(&self.n, k, v),
            size: self.size + 1,
        }
    }

    ///
    /// create and return a new map with the key, value pair removed
    ///
    pub fn remove(&self, k: K) -> Self {
        let size = match S::find(&self.n, k.clone()) {
            Some(_) => self.size - 1,
            None => self.size,
        };
        let n = S::remove(&self.n, k);
        Self { n, size }
    }

    ///
    /// search for a key and return true if the key exist, false otherwise
    ///
    pub fn exist(&self, k: K) -> bool {
        S::find(&self.n, k).is_some()
    }

    ///
    /// search for a key and return a pointer to the value if the key exists, None otherwise
    ///
    pub fn find(&self, k: K) -> Option<&V> {
        S::find(&self.n, k)
    }

    ///
    /// walk the list/stack and build a vector of keys and return it
    ///
    pub fn to_vec(&self) -> Vec<(K, V)> {
        let mut v = Vec::new();
        S::to_vec(&self.n, &mut v);
        v
    }

    ///
    /// return the maximum tree height
    ///
    pub fn height(&self) -> usize {
        self.n.height()
    }

    ///
    /// return true if the map is empty
    ///
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    ///
    /// return the number of elements in the map
    ///
    pub fn len(&self) -> usize {
        self.size
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
