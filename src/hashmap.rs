use std::marker::PhantomData;
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
use crate::{Hashable, TRIE_BITS, TRIE_MASK, TRIE_SIZE};
use std::mem::*;
use std::sync::Arc;

#[derive(Clone)]
enum HashMapNode<K: Hashable + Eq + Clone, V: Clone> {
    Empty,
    One(usize, K, V),
    Node(usize, Arc<[N<K, V>; TRIE_SIZE]>),
}

use HashMapNode::*;

type N<K, V> = HashMapNode<K, V>;
type H<K, V> = Arc<HashMapNode<K, V>>;

impl<K: Hashable + Eq + Clone, V: Clone> HashMapNode<K, V> {
    fn empty() -> H<K, V> {
        H::new(Empty)
    }

    fn new_empty_slice() -> [N<K, V>; TRIE_SIZE] {
        let mut s: [MaybeUninit<N<K, V>>; TRIE_SIZE] =
            unsafe { MaybeUninit::uninit().assume_init() };
        for i in s.iter_mut().take(TRIE_SIZE) {
            *i = MaybeUninit::new(N::Empty);
        }

        // TODO: issue: https://github.com/rust-lang/rust/issues/61956
        // use transmute
        // let ptr = &mut s as *mut _ as *mut [N<K, V>; TRIE_SIZE];
        // let res = unsafe { ptr.read() };
        // forget(s);
        // res

        unsafe {
            (&*(&MaybeUninit::new(s) as *const _ as *const MaybeUninit<_>)).assume_init_read()
        }
    }

    fn insert(h: &N<K, V>, l: u32, k: K, v: V) -> Option<N<K, V>> {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;

        match h {
            Empty => Some(N::One(kh, k, v)),
            One(hh, k2, _) if kh == *hh && k == *k2 =>
            /* (1) */
            {
                None
            }
            One(kh2, k2, v2) => {
                let mut slice = N::new_empty_slice();
                slice[idx] = N::One(kh, k, v);
                let idx2 = kh2.wrapping_shr(l) & TRIE_MASK;
                if idx2 != idx {
                    slice[idx2] = N::One(*kh2, k2.clone(), v2.clone());
                    let n = Node(2, Arc::new(slice));
                    Some(n)
                } else {
                    let n = Node(1, Arc::new(slice));
                    match N::insert(&n, l, k2.clone(), v2.clone()) {
                        Some(n2) => Some(n2), // return the new one
                        None => Some(n),      // this case should never be exausted: look at (1)
                    }
                }
            }
            Node(size, slice) => match N::insert(&slice[idx], l + TRIE_BITS, k, v) {
                None => None,
                Some(n) => {
                    let mut slice2 = slice.as_ref().clone();
                    slice2[idx] = n;
                    Some(Node(size + 1, Arc::new(slice2)))
                }
            },
        }
    }

    fn exist(h: &N<K, V>, l: u32, k: &K) -> bool {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;

        match h {
            Empty => false,
            One(hh, k2, _) => kh == *hh && k == k2,
            Node(_, slice) => N::exist(&slice[idx], l + TRIE_BITS, k),
        }
    }

    fn find(&self, l: u32, k: &K) -> Option<&V> {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;

        match self {
            Empty => None,
            One(hh, k2, v) if kh == *hh && k == k2 => Some(v),
            One(_, _, _) => None,
            Node(_, slice) => slice[idx].find(l + TRIE_BITS, k),
        }
    }

    fn remove(h: &N<K, V>, l: u32, k: K) -> Option<N<K, V>> {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;
        match h {
            Empty => None,
            One(hh, k2, _) if kh == *hh && k == *k2 =>
            /* (1) */
            {
                Some(Empty)
            }
            One(_, _, _) => None,
            Node(size, slice) => match N::remove(&slice[idx], l + TRIE_BITS, k) {
                None => None,
                Some(n) if matches!(n, Empty) && *size == 1 => Some(Empty),
                Some(n) => {
                    let new_size = match n {
                        Empty => size - 1,
                        _ => *size,
                    };
                    let mut slice2 = slice.as_ref().clone();
                    slice2[idx] = n;
                    Some(Node(new_size, Arc::new(slice2)))
                }
            },
        }
    }

    fn to_vec_internal(&self, v: &mut Vec<(K, V)>) {
        match self {
            Empty => (),
            One(_, k, vv) => v.push((k.clone(), vv.clone())),
            Node(_, slice) => {
                for n in slice.as_ref() {
                    n.to_vec_internal(v);
                }
            }
        }
    }

    fn to_vec(&self) -> Vec<(K, V)> {
        let mut v = Vec::new();
        self.to_vec_internal(&mut v);
        v
    }
}

#[derive(Clone)]
pub struct HashMap<K: Hashable + Eq + Clone, V: Clone> {
    n: H<K, V>,
    count: usize,
}

impl<K: Hashable + Eq + Clone, V: Clone> HashMap<K, V> {
    ///
    /// create and return a new empty map
    ///
    pub fn empty() -> Self {
        Self {
            n: N::empty(),
            count: 0,
        }
    }

    ///
    /// create and return a new map containing the new key, value pair
    ///
    pub fn insert(&self, k: K, v: V) -> Self {
        let n = N::insert(self.n.as_ref(), 0, k.clone(), v.clone());
        match n {
            Some(n) => Self {
                n: H::new(n),
                count: self.count + 1,
            },
            None => {
                // the key is already found, overwrite it
                let n = N::insert(self.remove(k.clone()).n.as_ref(), 0, k, v).unwrap();
                Self {
                    n: H::new(n),
                    count: self.count,
                }
            }
        }
    }

    ///
    /// create and return a new map with the key, value pair removed
    ///
    pub fn remove(&self, k: K) -> Self {
        let n = N::remove(self.n.as_ref(), 0, k);
        match n {
            Some(n) => Self {
                n: H::new(n),
                count: self.count - 1,
            },
            None => Self {
                n: self.n.clone(),
                count: self.count,
            },
        }
    }

    ///
    /// search for a key and return true if the key exist, false otherwise
    ///
    pub fn exist(&self, k: &K) -> bool {
        N::exist(self.n.as_ref(), 0, k)
    }

    ///
    /// search for a key and return a pointer to the value if the key exists, None otherwise
    ///
    pub fn find(&self, k: &K) -> Option<&V> {
        self.n.as_ref().find(0, k)
    }

    ///
    /// walk the list/stack and build a vector of keys and return it
    ///
    pub fn to_vec(&self) -> Vec<(K, V)> {
        self.n.to_vec()
    }

    ///
    /// return the number of elements in the set
    ///
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    ///
    /// return the number of elements in the set
    ///
    pub fn len(&self) -> usize {
        self.count
    }

    ///
    /// returns an iterator
    ///
    pub fn iter<'a>(&self) -> HMIter<'a, K, V> {
        HMIter {
            stack: Vec::new(),
            current: Pointer {
                node: self.n.clone(),
                idx: 0,
            },
            _phantom: PhantomData::default(),
        }
    }
}

#[derive(Clone)]
struct Pointer<K: Clone + Eq + Hashable, V: Clone> {
    idx: usize,
    node: H<K, V>,
}

pub struct HMIter<'a, K: Clone + Eq + Hashable, V: Clone> {
    stack: Vec<Pointer<K, V>>,
    current: Pointer<K, V>,
    _phantom: PhantomData<&'a (K, V)>,
}

impl<'a, K: Clone + Eq + Hashable, V: Clone> HMIter<'a, K, V> {
    fn pop(&mut self) {
        match self.stack.pop() {
            Some(Pointer { idx: i, node: n }) => {
                self.current = Pointer {
                    idx: i + 1,
                    node: n,
                }
            }

            None => {
                self.current = Pointer {
                    idx: 0,
                    node: Arc::new(HashMapNode::Empty),
                }
            }
        }
    }
}

impl<'a, K: Clone + Eq + Hashable, V: Clone> std::iter::Iterator for HMIter<'a, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let nc = self.current.clone(); // needless, but required for the borrow checker
        let n = nc.node.as_ref();
        match n {
            HashMapNode::Empty => {
                // we only enter this one if the root can be empty!
                None
            }

            HashMapNode::One(_s, k, v) => {
                // we only enter this one if it's in the root!
                if self.current.idx == 0 {
                    self.current.idx += 1;
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            }

            HashMapNode::Node(size, entries) => {
                while self.current.idx < TRIE_SIZE {
                    match &entries[self.current.idx] {
                        HashMapNode::Empty => self.current.idx += 1,
                        HashMapNode::One(_s, k, v) => {
                            self.current.idx += 1;
                            return Some((k.clone(), v.clone()));
                        }
                        HashMapNode::Node(new_size, new_entries) => {
                            self.stack.push(Pointer {
                                idx: self.current.idx,
                                node: Arc::new(HashMapNode::Node(*size, entries.clone())),
                            });
                            self.current = Pointer {
                                idx: 0,
                                node: Arc::new(HashMapNode::Node(*new_size, new_entries.clone())),
                            };
                            return self.next();
                        }
                    }
                }
                self.pop();
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hashmap::*;

    static mut SEED: usize = 777;

    fn rand() -> usize {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32 as usize
        }
    }

    #[test]
    fn insert() {
        let numbers = [3, 3, 0x13, 120, 4, 9, 27, 1, 45];
        let mut n = HashMap::empty();
        for i in numbers {
            n = n.insert(i, i * i);
        }

        assert_eq!(n.len(), 8);

        for i in 0..numbers.len() {
            assert_eq!(n.exist(&numbers[i]), true);
        }
    }

    #[test]
    fn insert_redundant_key_different_values() {
        let numbers = [3, 3, 0x13, 120, 4, 9, 27, 1, 45];
        let mut n = HashMap::empty();
        for i in numbers {
            n = n.insert(i, i * i);
        }

        for i in numbers {
            n = n.insert(i, 2 * i * i);
        }

        assert_eq!(n.len(), 8);

        for i in 0..numbers.len() {
            let num = numbers[i];
            assert!(n.exist(&num));
            assert_eq!(*n.find(&num).unwrap(), 2 * num * num);
        }
    }

    #[test]
    fn remove() {
        let numbers = [3, 3, 0x13, 120, 4, 9, 27, 1, 45];
        let mut n = HashMap::empty();
        for i in numbers {
            n = n.insert(i, i * i);
        }

        assert_eq!(n.len(), 8);

        for i in 0..numbers.len() {
            assert_eq!(n.exist(&numbers[i]), true);
        }

        for i in numbers {
            n = n.remove(i);
            assert_eq!(n.exist(&i), false);
        }
    }

    #[test]
    fn insert_1000000() {
        let mut numbers = Vec::new();
        let mut n = HashMap::empty();
        for _ in 0..1000000 {
            let r = rand() % 100000;
            n = n.insert(r, r * r);
            numbers.push(r);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(n.len(), sorted.len());

        for i in 0..numbers.len() {
            assert_eq!(n.exist(&numbers[i]), true);
            let k = numbers[i];

            assert_eq!(n.find(&k).is_some(), true);
            assert_eq!(*n.find(&k).unwrap(), k * k);
        }

        let mut v = n.to_vec();
        v.sort();
        assert_eq!(v.len(), sorted.len());
        for i in 0..sorted.len() {
            assert_eq!(sorted[i], v[i].0);
        }
    }

    #[test]
    fn remove_1000000() {
        let mut numbers = Vec::new();
        let mut n = HashMap::empty();
        for _ in 0..1000000 {
            let r = rand() % 100000;
            n = n.insert(r, r * r);
            numbers.push(r);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(n.len(), sorted.len());

        for i in 0..numbers.len() {
            assert_eq!(n.exist(&numbers[i]), true);
        }

        let mut v = n.to_vec();
        v.sort();
        assert_eq!(v.len(), sorted.len());
        for i in sorted {
            n = n.remove(i);
            assert_eq!(n.exist(&i), false);
        }

        assert_eq!(n.len(), 0);
    }

    #[test]
    fn iter_1000000() {
        let mut numbers = Vec::new();
        let mut n = HashMap::empty();
        for _ in 0..1000000 {
            let r = rand() % 100000;
            n = n.insert(r, r * r);
            numbers.push(r);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(n.len(), sorted.len());

        for i in 0..numbers.len() {
            assert_eq!(n.exist(&numbers[i]), true);
            let k = numbers[i];

            assert_eq!(n.find(&k).is_some(), true);
            assert_eq!(*n.find(&k).unwrap(), k * k);
        }

        let mut v = n.iter().collect::<Vec<_>>();
        v.sort();
        assert_eq!(v.len(), sorted.len());
        for i in 0..sorted.len() {
            assert_eq!(sorted[i], v[i].0);
        }
    }

    #[test]
    fn iter_1() {
        let mut n = HashMap::empty();
        n = n.insert(1, 1);
        for (k, v) in n.iter() {
            assert_eq!(k, 1);
            assert_eq!(v, 1);
        }
    }
}
