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
enum SetNode<K: Clone> {
    Empty,
    One(K),
    Node(usize, Arc<SetNode<K>>, K, Arc<SetNode<K>>),
}

use SetNode::*;
type N<K>   = Arc<SetNode<K>>;

fn empty    <K: Clone>()                                    -> N<K> { N::new(Empty) }
fn one      <K: Clone>(k: K)                                -> N<K> { N::new(One(k)) }
fn node     <K: Clone>(h: usize, l: &N<K>, k: K, r: &N<K>)  -> N<K> { N::new(Node(h, l.clone(), k, r.clone())) }

fn make<K: Clone>(l: &N<K>, k: K, r: &N<K>) -> N<K> {
    match (l.as_ref(), r.as_ref()) {
        (Empty, Empty) => one(k),
        _ => {
            let h   = 1 + usize::max(l.height(), r.height());
            node(h, l, k, r)
        }
    }
}

fn rebalance<K: Clone>(t1: &N<K>, k: K, t2: &N<K>) -> N<K> {
    let t1h = t1.height();
    let t2h = t2.height();

    if t2h > t1h + 2 {
        match t2.as_ref() {
            Node(_, t2l, t2x, t2r) => {
                if t2l.height() > t1h + 1 {
                    match t2l.as_ref() {
                        Node(_, t2ll, t2lx, t2lr) => make(&make(t1, k, t2ll), t2lx.clone(), &make(t2lr, t2x.clone(), t2r)),
                        _ => unreachable!()
                    }
                } else {
                    make(&make(t1, k, t2l), t2x.clone(), t2r)
                }
            },
            _ => unreachable!()
        }
    } else {
        if t1h > t2h + 2 {
            match t1.as_ref() {
                Node(_, t1l, t1x, t1r) => {
                    if t1r.height() > t2h + 1 {
                        match t1r.as_ref() {
                            Node(_, t1rl, t1rx, t1rr) => make(&make(t1l, t1x.clone(), t1rl), t1rx.clone(), &make(t1rr, k, t2)),
                            _ => unreachable!()
                        }
                    } else {
                        make(t1l, t1x.clone(), &make(t1r, k, t2))
                    }
                },
                _ => unreachable!()
            }
        } else {
            make(t1, k, t2)
        }
    }
}

fn insert<K: Ord + Clone>(t: &N<K>, k: K) -> N<K> {
    match t.as_ref() {
        Node(_, l, k2, r)   if k < k2.clone()   => rebalance(&insert(l, k), k2.clone(), r),
        Node(h, l, k2, r)   if k == k2.clone()  => node(*h, l, k2.clone(), r),
        Node(_, l, k2, r)   if k > k2.clone()   => rebalance(l, k2.clone(), &insert(r, k)),

        One(k2)             if k < k2.clone()   => node(2, &empty(), k, &one(k2.clone())),
        One(k2)             if k == k2.clone()  => one(k2.clone()),
        One(k2)             if k > k2.clone()   => node(2, &one(k2.clone()), k, &empty()),

        Empty                           => one(k),
        _                               => unreachable!()
    }
}

fn splice_out_successor<K: Clone>(t: &N<K>) -> (K, N<K>) {
    match t.as_ref() {
        Empty   => panic!("internal error"),
        One(k2) => (k2.clone(), empty()),
        Node(_, l, k2, r) => {
            let l1 = l.clone();
            let r1 = r.clone();
            match l.as_ref() {
                Empty   => (k2.clone(), r1),
                _ => {
                    let (x3, ll) = splice_out_successor(&l1);
                    (x3, make(&ll, k2.clone(), r))
                }
            }
        }
    }
}

fn remove<K: Ord + Clone>(t: &N<K>, k: K) -> N<K> {
    match t.as_ref() {
        Empty                                   => empty(),
        One(k2)             if k == k2.clone()  => empty(),
        One(k2)                                 => one(k2.clone()),
        Node(_, l, k2, r)   if k < k2.clone()   => rebalance(&remove(l, k), k2.clone(), r),
        Node(_, l, k2, r)   if k == k2.clone()  => {
            let l1 = l.clone();
            let r1 = r.clone();
            match (l.as_ref(), r.as_ref()) {
                (Empty, _)  => r1,
                (_, Empty)  => l1,
                _           => {
                    let (sx, rr) = splice_out_successor(&r1);
                    make(&l1, sx, &rr)
                }
            }
        },
        Node(_, l, k2, r)   if k > k2.clone()   => rebalance(l, k2.clone(), &remove(r, k)),
        _ => unreachable!()
    }
}

fn find<K: Ord + Clone>(t: &N<K>, k: K) -> Option<&N<K>> {
    match t.as_ref() {
        Empty                       => None,
        One(k2) if k == k2.clone()  => Some(t),
        One(_)                      => None,
        Node(_, l, k2, _)   if k < k2.clone()   => find(l, k),
        Node(_, _, k2, _)   if k == k2.clone()  => Some(t),
        Node(_, _, k2, r)   if k > k2.clone()   => find(r, k),
        _                           => unreachable!()
    }
}

fn to_vec<K: Ord + Clone>(t: &N<K>, v: &mut Vec<K>) {
    match t.as_ref() {
        Empty                   => (),
        One(k)                  => v.push(k.clone()),
        Node(_, l, k, r)        => {
            to_vec(l, v);
            v.push(k.clone());
            to_vec(r, v);
        }
    }
}

impl<K : Clone> SetNode<K> {
    fn height(&self) -> usize {
        match self {
            Empty               => 0,
            One(_)              => 1,
            Node(h, _, _, _)    => *h
        }
    }
}

pub struct Set<K: Ord + Clone> {
    size    : usize,
    n       : N<K>,
}

impl<K: Ord + Clone> Set<K> {
    ///
    /// create and return a new empty set
    ///
    pub fn empty()              -> Self { Self { n: empty(), size: 0 } }

    ///
    /// insert a new key and return a new set with the new element added to it
    ///
    pub fn insert(&self, k: K)  -> Self { Self { n: insert(&self.n, k), size: self.size + 1 } }

    ///
    /// remove a key and return a new set with the element removed to it
    ///
    pub fn remove(&self, k: K)  -> Self {
        let size = match find(&self.n, k.clone()) {
            Some(_)     => self.size - 1,
            None        => self.size
        };
        let n = remove(&self.n, k);
        Self { n, size }
    }

    ///
    /// search for a key and return true if the key exist, false otherwise
    ///
    pub fn exist(&self, k: K)    -> bool {
        let n = find(&self.n, k);
        match n {
            Some(_)     => true,
            None        => false
        }
    }

    ///
    /// walk the list/stack and build a vector of keys and return it
    ///
    pub fn to_vec(&self)        -> Vec<K> {
        let mut v   = Vec::new();
        to_vec(&self.n, &mut v); v
    }

    ///
    /// return the maximum tree height
    ///
    pub fn height(&self)        -> usize { self.n.height() }

    ///
    /// return the number of elements in the set
    ///
    pub fn len(&self)           -> usize { self.size }
}

#[cfg(test)]
mod tests {
    use crate::set::*;

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
        let mut n   = Set::empty();
        for i in numbers {
            n   = n.insert(i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i], sorted[i]);
        }
    }

    #[test]
    fn find() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let mut n   = Set::empty();
        for i in numbers {
            n   = n.insert(i);
        }

        assert_eq!(n.exist(10), true);
        assert_eq!(n.exist(11), true);
    }

    #[test]
    fn remove() {
        let numbers = [5, 10, 3, 120, 4, 9, 27, 1, 45];
        let sorted  = [1, 3, 4, 9, 10, 27, 45, 120];
        let mut n   = Set::empty();
        for i in numbers {
            n   = n.insert(i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), numbers.len());

        n = n.remove(5);

        let v = n.to_vec();

        for i in 0..v.len() {
            assert_eq!(v[i], sorted[i]);
        }
    }

    #[test]
    fn remove_one_from_one() {
        let mut n   = Set::empty();
        n   = n.insert(10);

        assert_eq!(n.exist(5), false);
        n   = n.remove(5);

        assert_eq!(n.exist(10), false);
        n   = n.remove(10);
        assert_eq!(n.exist(10), false);

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
        let mut n   = Set::empty();
        for i in numbers {
            n   = n.insert(i);
        }

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i], sorted[i]);
        }
    }

    #[test]
    fn remove_5000_from_10000_random() {
        let mut hs      = std::collections::hash_set::HashSet::new();
        let mut numbers = Vec::new();
        for _ in 0..10000 {
            hs.insert(rand() % 10000);
        }

        for i in hs.iter() {
            numbers.push(*i);
        }

        let mut n   = Set::empty();
        for i in numbers.iter() {
            n   = n.insert(*i);
        }

        assert_eq!(n.len(), hs.len());

        let mut hs = hs.clone();

        for i in 0..hs.len() / 2 {
            hs.remove(&numbers[i]);
            n = n.remove(numbers[i]);
        }

        assert_eq!(n.len(), hs.len());

        let mut sorted  = Vec::new();
        for i in hs.iter() {
            sorted.push(*i);
        }
        sorted.sort();

        let v = n.to_vec();

        assert_eq!(v.len(), sorted.len());

        for i in 0..v.len() {
            assert_eq!(v[i], sorted[i]);
        }

        assert_eq!(n.exist(numbers[0]), false);
        assert_eq!(n.to_vec().len(), hs.len());
    }
}
