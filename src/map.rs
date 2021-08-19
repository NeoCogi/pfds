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
enum MapNode<K: Clone, V> {
    Empty,
    One(K, Arc<V>),
    Node(usize, Arc<MapNode<K, V>>, K, Arc<V>, Arc<MapNode<K, V>>),
}

use MapNode::*;
type N<K, V>   = Arc<MapNode<K, V>>;

fn empty    <K: Clone, V>()                                    -> N<K, V> { N::new(Empty) }
fn one      <K: Clone, V>(k: K, v: &Arc<V>)                    -> N<K, V> { N::new(One(k, v.clone())) }
fn node     <K: Clone, V>(h: usize, l: &N<K, V>, k: K, v: &Arc<V>, r: &N<K, V>)    -> N<K, V> { N::new(Node(h, l.clone(), k, v.clone(), r.clone())) }

fn make<K: Clone, V>(l: &N<K, V>, k: K, v: &Arc<V>, r: &N<K, V>) -> N<K, V> {
    match (l.as_ref(), r.as_ref()) {
        (Empty, Empty) => one(k, v),
        _ => {
            let h   = 1 + usize::max(l.height(), r.height());
            node(h, l, k, v, r)
        }
    }
}

fn rebalance<K: Clone, V>(t1: &N<K, V>, k: K, v: &Arc<V>, t2: &N<K, V>) -> N<K, V> {
    let t1h = t1.height();
    let t2h = t2.height();

    if t2h > t1h + 2 {
        match t2.as_ref() {
            Node(_, t2l, t2k, t2v, t2r) => {
                if t2l.height() > t1h + 1 {
                    match t2l.as_ref() {
                        Node(_, t2ll, t2lk, t2lv, t2lr) => make(&make(t1, k, v, t2ll), t2lk.clone(), t2lv, &make(t2lr, t2k.clone(), t2v, t2r)),
                        _ => unreachable!()
                    }
                } else {
                    make(&make(t1, k, v, t2l), t2k.clone(), t2v, t2r)
                }
            },
            _ => unreachable!()
        }
    } else {
        if t1h > t2h + 2 {
            match t1.as_ref() {
                Node(_, t1l, t1k, t1v, t1r) => {
                    if t1r.height() > t2h + 1 {
                        match t1r.as_ref() {
                            Node(_, t1rl, t1rk, t1rv, t1rr) => make(&make(t1l, t1k.clone(), t1v, t1rl), t1rk.clone(), t1rv, &make(t1rr, k, v, t2)),
                            _ => unreachable!()
                        }
                    } else {
                        make(t1l, t1k.clone(), t1v, &make(t1r, k, v, t2))
                    }
                },
                _ => unreachable!()
            }
        } else {
            make(t1, k, v, t2)
        }
    }
}

fn insert<K: Ord + Clone, V>(t: &N<K, V>, k: K, v: &Arc<V>) -> N<K, V> {
    match t.as_ref() {
        Node(_, l, k2, v2, r)   if k < k2.clone()   => rebalance(&insert(l, k, v), k2.clone(), v2, r),
        Node(h, l, k2, v2, r)   if k == k2.clone()  => node(*h, l, k2.clone(), v2, r),
        Node(_, l, k2, v2, r)   if k > k2.clone()   => rebalance(l, k2.clone(), v2, &insert(r, k, v)),

        One(k2, v2)             if k < k2.clone()   => node(2, &empty(), k, v, &one(k2.clone(), v2)),
        One(k2, v2)             if k == k2.clone()  => one(k2.clone(), v2),
        One(k2, v2)             if k > k2.clone()   => node(2, &one(k2.clone(), v2), k, v, &empty()),

        Empty                                       => one(k, v),
        _                                           => unreachable!()
    }
}

fn splice_out_successor<K: Clone, V>(t: &N<K, V>) -> (K, Arc<V>, N<K, V>) {
    match t.as_ref() {
        Empty   => panic!("internal error"),
        One(k2, v2) => (k2.clone(), v2.clone(), empty()),
        Node(_, l, k2, v2, r) => {
            let l1 = l.clone();
            let r1 = r.clone();
            match l.as_ref() {
                Empty   => (k2.clone(), v2.clone(), r1),
                _ => {
                    let (k3, v3, ll) = splice_out_successor(&l1);
                    (k3, v3.clone(), make(&ll, k2.clone(), v2, r))
                }
            }
        }
    }
}

fn remove<K: Ord + Clone, V>(t: &N<K, V>, k: K) -> N<K, V> {
    match t.as_ref() {
        Empty                                   => empty(),
        One(k2, _)              if k == k2.clone()  => empty(),
        One(k2, v2)                                 => one(k2.clone(), v2),
        Node(_, l, k2, v2, r)   if k < k2.clone()   => rebalance(&remove(l, k), k2.clone(), v2, r),
        Node(_, l, k2, _, r)    if k == k2.clone()  => {
            let l1 = l.clone();
            let r1 = r.clone();
            match (l.as_ref(), r.as_ref()) {
                (Empty, _)  => r1,
                (_, Empty)  => l1,
                _           => {
                    let (sk, sv, rr) = splice_out_successor(&r1);
                    make(&l1, sk, &sv, &rr)
                }
            }
        },
        Node(_, l, k2, v2, r)   if k > k2.clone()   => rebalance(l, k2.clone(), v2, &remove(r, k)),
        _ => unreachable!()
    }
}

fn find<K: Ord + Clone, V>(t: &N<K, V>, k: K) -> Option<&N<K, V>> {
    match t.as_ref() {
        Empty                       => None,
        One(k2, _) if k == k2.clone()  => Some(t),
        One(_, _)                      => None,
        Node(_, l, k2, _, _)   if k < k2.clone()   => find(l, k),
        Node(_, _, k2, _, _)   if k == k2.clone()  => Some(t),
        Node(_, _, k2, _, r)   if k > k2.clone()   => find(r, k),
        _                           => unreachable!()
    }
}

fn to_vec<K: Ord + Clone, V>(t: &N<K, V>, vec: &mut Vec<(K, Arc<V>)>) {
    match t.as_ref() {
        Empty                   => (),
        One(k, v)                  => vec.push((k.clone(), v.clone())),
        Node(_, l, k, v, r)        => {
            to_vec(l, vec);
            vec.push((k.clone(), v.clone()));
            to_vec(r, vec);
        }
    }
}

impl<K : Clone, V> MapNode<K, V> {
    fn height(&self) -> usize {
        match self {
            Empty               => 0,
            One(_, _)           => 1,
            Node(h, _, _, _, _) => *h
        }
    }
}

pub struct Map<K: Ord + Clone, V> {
    size: usize,
    n   : N<K, V>,
}

impl<K: Ord + Clone, V> Map<K, V> {
    pub fn empty()              -> Self { Self { n: empty(), size: 0 } }
    pub fn insert(&self, k: K, v: V)  -> Self { Self { n: insert(&self.n, k, &Arc::new(v)), size: self.size + 1 } }
    pub fn remove(&self, k: K)  -> Self {
        let size = match find(&self.n, k.clone()) {
            Some(_)     => self.size - 1,
            None        => self.size
        };
        let n = remove(&self.n, k);
        Self { n, size }
    }
    pub fn find(&self, k: K)    -> Option<Self> {
        let n = find(&self.n, k);
        match n {
            Some(n)     => Some(Self { n: n.clone(), size: self.size }),
            None        => None
        }
    }
    pub fn to_vec(&self)        -> Vec<(K, Arc<V>)> {
        let mut v   = Vec::new();
        to_vec(&self.n, &mut v); v
    }

    pub fn height(&self)        -> usize { self.n.height() }
    pub fn size(&self)          -> usize { self.size }
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
        let sorted  = [1, 3, 4, 5, 9, 10, 27, 45, 120];
        let mut n   = Map::empty();
        for i in numbers {
            n   = n.insert(i, i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(*v[i].1, sorted[i]);
        }
    }

    #[test]
    fn find() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let mut n   = Map::empty();
        for i in numbers {
            n   = n.insert(i, i);
        }

        assert_eq!(n.find(10).is_some(), true);
        assert_eq!(n.find(11).is_none(), true);
    }

    #[test]
    fn remove() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let sorted  = [1, 3, 4, 9, 10, 27, 45, 120];
        let mut n   = Map::empty();
        for i in numbers {
            n   = n.insert(i, i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), numbers.len());

        n = n.remove(5);

        let v = n.to_vec();

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(*v[i].1, sorted[i]);
        }
    }

    #[test]
    fn remove_one_from_one() {
        let mut n   = Map::empty();
        n   = n.insert(10, 10);

        assert_eq!(n.find(5).is_none(), true);
        n   = n.remove(5);

        assert_eq!(n.find(10).is_some(), true);
        n   = n.remove(10);
        assert_eq!(n.find(10).is_none(), true);

        let v = n.to_vec();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn insert_10000_random() {
        let mut hs      = std::collections::hash_set::HashSet::new();
        let mut numbers = Vec::new();
        for _ in 0..10000 {
            hs.insert(rand());
        }

        for i in hs.iter() {
            numbers.push(i);
        }

        let mut sorted  = numbers.clone();
        sorted.sort();
        let mut n   = Map::empty();
        for i in numbers {
            n   = n.insert(i, i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(*v[i].1, sorted[i]);
        }
    }

    #[test]
    fn remove_5000_from_10000_random() {
        let mut hs      = std::collections::hash_set::HashSet::new();
        let mut numbers = Vec::new();
        for _ in 0..10000 {
            hs.insert(rand());
        }

        for i in hs.iter() {
            numbers.push(i);
        }

        let mut n   = Map::empty();
        let numbers2 = numbers.clone();
        for i in numbers {
            n   = n.insert(i, i);
        }

        let mut hs = hs.clone();

        for i in 0..5000 {
            hs.remove(numbers2[i]);
            n = n.remove(numbers2[i]);
        }

        let mut sorted  = Vec::new();
        for i in hs.iter() {
            sorted.push(i);
        }
        sorted.sort();

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i].0, sorted[i]);
            assert_eq!(*v[i].1, sorted[i]);
        }

        assert_eq!(n.find(numbers2[0]).is_none(), true);
        assert_eq!(n.to_vec().len(), 5000);
    }
}
