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
use std::sync::Arc;

enum ListNode<E> {
    Nil,
    Node(usize, E, Arc<ListNode<E>>),
}

use ListNode::*;

type N<E> = Arc<ListNode<E>>;

fn empty<E>() -> N<E> {
    Arc::new(Nil)
}
fn node<E>(s: usize, e: E, l: &N<E>) -> N<E> {
    Arc::new(Node(s, e, l.clone()))
}

fn push<E>(l: &N<E>, e: E) -> N<E> {
    match l.as_ref() {
        Nil => node(1, e, l),
        Node(s, _, _) => node(s + 1, e, l),
    }
}

fn pop<E>(l: &N<E>) -> N<E> {
    match l.as_ref() {
        Nil => panic!("pop: list is empty"),
        Node(_, _, l) => l.clone(),
    }
}

fn top<E>(l: &N<E>) -> &E {
    match l.as_ref() {
        Nil => panic!("top: list is empty"),
        Node(_, e, _) => e,
    }
}

fn len<E>(l: &N<E>) -> usize {
    match l.as_ref() {
        Nil => 0,
        Node(s, _, _) => *s,
    }
}

fn to_vec<E: Clone>(l: &N<E>) -> Vec<E> {
    let mut v = Vec::new();
    let mut n = l;
    loop {
        match n.as_ref() {
            Nil => return v,
            Node(_, e, l) => {
                v.push(e.clone());
                n = l;
            }
        }
    }
}

fn rev<E: Clone>(l: &N<E>) -> N<E> {
    let mut n = empty();
    let mut s = l;
    loop {
        match s.as_ref() {
            Nil => return n,
            Node(_, e, l) => {
                n = push(&n, e.clone());
                s = l;
            }
        }
    }
}

/// A persistent (immutable) stack/list data structure.
/// 
/// `List` is implemented as a singly-linked list with structural sharing,
/// making it efficient for stack-like operations. All operations return
/// a new list, leaving the original unchanged.
/// 
/// # Performance
/// 
/// - `push`: O(1)
/// - `pop`: O(1)
/// - `top`: O(1)
/// - `len`: O(1) - length is cached
/// - `rev`: O(n)
/// - `to_vec`: O(n)
#[derive(Clone)]
pub struct List<E: Clone + Sized> {
    n: N<E>,
}

impl<E: Clone + Sized> List<E> {
    /// Creates a new empty list.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list: List<i32> = List::empty();
    /// assert!(list.is_empty());
    /// assert_eq!(list.len(), 0);
    /// ```
    pub fn empty() -> Self {
        Self { n: empty() }
    }

    /// Creates a new list with the given element added to the front (top).
    /// 
    /// This operation is O(1) and shares structure with the original list.
    /// 
    /// # Arguments
    /// 
    /// * `e` - The element to add to the front of the list
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(1).push(2).push(3);
    /// assert_eq!(list.len(), 3);
    /// assert_eq!(*list.top(), 3); // Last pushed element is at the top
    /// ```
    pub fn push(&self, e: E) -> Self {
        Self {
            n: push(&self.n, e),
        }
    }

    /// Creates a new list with the top element removed.
    /// 
    /// This operation is O(1) and shares structure with the original list.
    /// 
    /// # Panics
    /// 
    /// Panics if the list is empty.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(1).push(2);
    /// let list2 = list.pop();
    /// assert_eq!(list.len(), 2);  // Original unchanged
    /// assert_eq!(list2.len(), 1);
    /// assert_eq!(*list2.top(), 1);
    /// ```
    pub fn pop(&self) -> Self {
        Self { n: pop(&self.n) }
    }

    /// Returns a reference to the top element of the list.
    /// 
    /// This operation is O(1).
    /// 
    /// # Panics
    /// 
    /// Panics if the list is empty.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(42);
    /// assert_eq!(*list.top(), 42);
    /// ```
    pub fn top(&self) -> &E {
        top(&self.n)
    }

    /// Returns true if the list is empty.
    /// 
    /// This operation is O(1).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let empty = List::<i32>::empty();
    /// assert!(empty.is_empty());
    /// 
    /// let non_empty = empty.push(1);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        len(&self.n) == 0
    }

    /// Returns the number of elements in the list.
    /// 
    /// This operation is O(1) as the length is cached.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(1).push(2).push(3);
    /// assert_eq!(list.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        len(&self.n)
    }

    /// Converts the list to a vector.
    /// 
    /// Elements are returned with the top element first.
    /// This operation is O(n).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(1).push(2).push(3);
    /// let vec = list.to_vec();
    /// assert_eq!(vec, vec![3, 2, 1]); // Top element (3) is first
    /// ```
    pub fn to_vec(&self) -> Vec<E> {
        to_vec(&self.n)
    }

    /// Creates a new list that is the reverse of the current list.
    /// 
    /// This operation is O(n).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(1).push(2).push(3);
    /// let reversed = list.rev();
    /// assert_eq!(reversed.to_vec(), vec![1, 2, 3]);
    /// ```
    pub fn rev(&self) -> List<E> {
        List { n: rev(&self.n) }
    }

    /// Returns an iterator over the list elements.
    /// 
    /// The iterator yields elements from top to bottom.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::List;
    /// 
    /// let list = List::empty().push(1).push(2).push(3);
    /// let collected: Vec<_> = list.iter().collect();
    /// assert_eq!(collected, vec![3, 2, 1]);
    /// ```
    pub fn iter<'a>(&self) -> Iter<'a, E> {
        Iter {
            node: self.n.clone(),
            _phantom: PhantomData::default(),
        }
    }
}

fn drop_next<E>(n: &mut N<E>) -> Option<N<E>> {
    let mv = N::get_mut(n);
    match mv {
        None => None,
        Some(Nil) => None,
        Some(v) => {
            let old_v = std::mem::replace(v, Nil);
            match old_v {
                Nil => None,
                Node(_, _, l) => Some(l),
            }
        }
    }
}

impl<E: Clone> Drop for List<E> {
    fn drop(&mut self) {
        let mut n = drop_next(&mut self.n);
        loop {
            match &mut n {
                Some(v) => n = drop_next(v),
                None => return,
            }
        }
    }
}

pub struct Iter<'a, E> {
    node: N<E>,
    _phantom: PhantomData<&'a E>,
}

impl<'a, E: Clone> std::iter::Iterator for Iter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let nc = self.node.clone(); // needless, but required for the borrow checker
        let n = nc.as_ref();
        match n {
            ListNode::Nil => None,
            ListNode::Node(_, e, next) => {
                self.node = next.clone();
                Some(e.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::list::*;

    static mut SEED: i64 = 777;

    fn rand() -> i32 {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32
        }
    }

    #[test]
    fn push() {
        let mut elements = Vec::new();
        let mut l = List::empty();
        for _ in 0..1000 {
            let e = rand();
            elements.push(e);
            l = l.push(e);
        }

        assert_eq!(elements.len(), 1000);
        assert_eq!(elements.len(), l.len());

        let list_elems = l.to_vec();
        for i in 0..1000 {
            assert_eq!(list_elems[l.len() - 1 - i], elements[i]);
        }
    }

    #[test]
    fn pop() {
        let mut elements = Vec::new();
        let mut l = List::empty();
        for _ in 0..100000 {
            let e = rand();
            elements.push(e);
            l = l.push(e);
        }

        assert_eq!(elements.len(), 100000);
        assert_eq!(elements.len(), l.len());

        let list_elems = l.to_vec();
        for i in 0..100000 {
            assert_eq!(list_elems[l.len() - 1 - i], elements[i]);
        }

        let orig_len = l.len();
        for i in 0..50000 {
            let e = l.top();
            let e2 = elements[orig_len - 1 - i];
            assert_eq!(*e, e2);
            l = l.pop();
        }

        assert_eq!(l.len(), 50000);

        let list_elems = l.to_vec();
        for i in 0..50000 {
            assert_eq!(list_elems[l.len() - 1 - i], elements[i]);
        }
    }

    #[test]
    fn iter() {
        let mut elements = Vec::new();
        let mut l = List::empty();
        for _ in 0..1000 {
            let e = rand();
            elements.push(e);
            l = l.push(e);
        }

        assert_eq!(elements.len(), 1000);
        assert_eq!(elements.len(), l.len());

        let mut count = 0;
        for i in l.iter() {
            assert_eq!(i, elements[elements.len() - count - 1]);
            count += 1;
        }
    }
}
