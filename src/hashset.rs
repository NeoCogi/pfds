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
enum HashSetNode<K: Hashable + Eq + Clone> {
    Empty,
    One(usize, K),
    Node(usize, Arc<[N<K>; TRIE_SIZE]>),
}

use HashSetNode::*;

type N<K> = HashSetNode<K>;
type H<K> = Arc<HashSetNode<K>>;

impl<K: Hashable + Eq + Clone> HashSetNode<K> {
    fn empty() -> H<K> {
        H::new(Empty)
    }

    fn new_empty_slice() -> [N<K>; TRIE_SIZE] {
        let mut s: [MaybeUninit<N<K>>; TRIE_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in s.iter_mut().take(TRIE_SIZE) {
            *i = MaybeUninit::new(N::Empty);
        }
        //
        // // TODO: issue: https://github.com/rust-lang/rust/issues/61956
        // // use transmute
        //
        // let ptr = &mut s as *mut _ as *mut [N<K>; TRIE_SIZE];
        // let res = unsafe { ptr.read() };
        // forget(s);
        // res

        unsafe {
            (&*(&MaybeUninit::new(s) as *const _ as *const MaybeUninit<_>)).assume_init_read()
        }
    }

    fn insert(h: &N<K>, l: u32, k: K) -> Option<N<K>> {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;

        match h {
            Empty => Some(N::One(kh, k)),
            One(hh, k2) if kh == *hh && k == *k2 =>
            /* (1) */
            {
                None
            }
            One(kh2, k2) => {
                let mut slice = N::new_empty_slice();
                slice[idx] = N::One(kh, k);
                let idx2 = kh2.wrapping_shr(l) & TRIE_MASK;
                if idx2 != idx {
                    slice[idx2] = N::One(*kh2, k2.clone());
                    let n = Node(2, Arc::new(slice));
                    Some(n)
                } else {
                    let n = Node(1, Arc::new(slice));
                    match N::insert(&n, l, k2.clone()) {
                        Some(n2) => Some(n2), // return the new one
                        None => Some(n),      // this case should never be exhausted: look at (1)
                    }
                }
            }
            Node(size, slice) => match N::insert(&slice[idx], l + TRIE_BITS, k) {
                None => None,
                Some(n) => {
                    let mut slice2 = slice.as_ref().clone();
                    slice2[idx] = n;
                    Some(Node(size + 1, Arc::new(slice2)))
                }
            },
        }
    }

    fn exist(h: &N<K>, l: u32, k: K) -> bool {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;

        match h {
            Empty => false,
            One(hh, k2) => kh == *hh && k == *k2,
            Node(_, slice) => N::exist(&slice[idx], l + TRIE_BITS, k),
        }
    }

    fn remove(h: &N<K>, l: u32, k: K) -> Option<N<K>> {
        let kh = k.hash() as usize;
        let idx = kh.wrapping_shr(l) & TRIE_MASK;
        match h {
            Empty => None,
            One(hh, k2) if kh == *hh && k == *k2 =>
            /* (1) */
            {
                Some(Empty)
            }
            One(_, _) => None,
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

    fn to_vec_internal(&self, v: &mut Vec<K>) {
        match self {
            Empty => (),
            One(_, k) => v.push(k.clone()),
            Node(_, slice) => {
                for n in slice.as_ref() {
                    n.to_vec_internal(v);
                }
            }
        }
    }

    fn to_vec(&self) -> Vec<K> {
        let mut v = Vec::new();
        self.to_vec_internal(&mut v);
        v
    }
}

#[derive(Clone)]
pub struct HashSet<K: Hashable + Eq + Clone> {
    n: H<K>,
    count: usize,
}

impl<K: Hashable + Eq + Clone> HashSet<K> {
    ///
    /// create and return a new empty set
    ///
    pub fn empty() -> Self {
        Self {
            n: N::empty(),
            count: 0,
        }
    }

    ///
    /// insert a new key and return a new set with the new element added to it
    ///
    pub fn insert(&self, k: K) -> Self {
        let n = N::insert(self.n.as_ref(), 0, k.clone());
        match n {
            Some(n) => Self {
                n: H::new(n),
                count: self.count + 1,
            },
            None => {
                // the key is already found, return self unchanged
                self.clone()
            }
        }
    }

    ///
    /// remove a key and return a new set with the element removed to it
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
    /// walk the list/stack and build a vector of keys and return it
    ///
    pub fn exist(&self, k: K) -> bool {
        N::exist(self.n.as_ref(), 0, k)
    }

    pub fn to_vec(&self) -> Vec<K> {
        self.n.to_vec()
    }

    ///
    /// return true if the set is empty
    ///
    pub fn is_true(&self) -> bool {
        self.count == 0
    }

    ///
    /// return true if the set is empty
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
    pub fn iter<'a>(&self) -> HSIter<'a, K> {
        HSIter {
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
struct Pointer<E: Clone + Eq + Hashable> {
    idx: usize,
    node: H<E>,
}

pub struct HSIter<'a, E: Clone + Eq + Hashable> {
    stack: Vec<Pointer<E>>,
    current: Pointer<E>,
    _phantom: PhantomData<&'a E>,
}

impl<'a, E: Clone + Eq + Hashable> HSIter<'a, E> {
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
                    node: Arc::new(HashSetNode::Empty),
                }
            }
        }
    }
}

impl<'a, E: Clone + Eq + Hashable> std::iter::Iterator for HSIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let nc = self.current.clone(); // needless, but required for the borrow checker
        let n = nc.node.as_ref();
        match n {
            HashSetNode::Empty => {
                // we only enter this one if the root can be empty!
                None
            }

            HashSetNode::One(_k, v) => {
                // we only enter this one if it's in the root!
                if self.current.idx == 0 {
                    self.current.idx += 1;
                    Some(v.clone())
                } else {
                    None
                }
            }

            HashSetNode::Node(size, entries) => {
                while self.current.idx < TRIE_SIZE {
                    match &entries[self.current.idx] {
                        HashSetNode::Empty => self.current.idx += 1,
                        HashSetNode::One(_k, v) => {
                            self.current.idx += 1;
                            return Some(v.clone());
                        }
                        HashSetNode::Node(new_size, new_entries) => {
                            self.stack.push(Pointer {
                                idx: self.current.idx,
                                node: Arc::new(HashSetNode::Node(*size, entries.clone())),
                            });
                            self.current = Pointer {
                                idx: 0,
                                node: Arc::new(HashSetNode::Node(*new_size, new_entries.clone())),
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
    use crate::hashset::*;

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
        let mut n = HashSet::empty();
        for i in numbers {
            n = n.insert(i);
        }

        assert_eq!(n.len(), 8);

        for i in 0..numbers.len() {
            assert_eq!(n.exist(numbers[i]), true);
        }
    }

    #[test]
    fn remove() {
        let numbers = [3, 3, 0x13, 120, 4, 9, 27, 1, 45];
        let mut n = HashSet::empty();
        for i in numbers {
            n = n.insert(i);
        }

        assert_eq!(n.len(), 8);

        for i in 0..numbers.len() {
            assert_eq!(n.exist(numbers[i]), true);
        }

        for i in numbers {
            n = n.remove(i);
            assert_eq!(n.exist(i), false);
        }
    }

    #[test]
    fn insert_1000000() {
        let mut numbers = Vec::new();
        let mut n = HashSet::empty();
        for _ in 0..1000000 {
            let r = rand() % 100000;
            n = n.insert(r);
            numbers.push(r);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(n.len(), sorted.len());

        for i in 0..numbers.len() {
            assert_eq!(n.exist(numbers[i]), true);
        }

        let mut v = n.to_vec();
        v.sort();
        assert_eq!(v.len(), sorted.len());
        for i in 0..sorted.len() {
            assert_eq!(sorted[i], v[i]);
        }
    }

    #[test]
    fn remove_1000000() {
        let mut numbers = Vec::new();
        let mut n = HashSet::empty();
        for _ in 0..1000000 {
            let r = rand() % 100000;
            n = n.insert(r);
            numbers.push(r);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(n.len(), sorted.len());

        for i in 0..numbers.len() {
            assert_eq!(n.exist(numbers[i]), true);
        }

        let mut v = n.to_vec();
        v.sort();
        assert_eq!(v.len(), sorted.len());
        for i in sorted {
            n = n.remove(i);
            assert_eq!(n.exist(i), false);
        }

        assert_eq!(n.len(), 0);
    }

    #[test]
    fn iter_1000000() {
        let mut numbers = Vec::new();
        let mut n = HashSet::empty();
        for _ in 0..1000000 {
            let r = rand() % 100000;
            n = n.insert(r);
            numbers.push(r);
        }

        let mut sorted = numbers.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(n.len(), sorted.len());

        for i in 0..numbers.len() {
            assert_eq!(n.exist(numbers[i]), true);
        }

        let mut v = n.iter().collect::<Vec<_>>();
        v.sort();
        assert_eq!(v.len(), sorted.len());
        for i in 0..sorted.len() {
            assert_eq!(sorted[i], v[i]);
        }
    }

    #[test]
    fn iter_1() {
        let mut n = HashSet::empty();
        n = n.insert(1);
        for c in n.iter() {
            assert_eq!(c, 1);
        }
    }
}
