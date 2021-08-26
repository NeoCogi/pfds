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

use crate::list::*;

enum QueueNode<E: Clone> {
    Empty,
    Node { back: L<E>, front: L<E> }
}

use QueueNode::*;

type N<E>   = Arc<QueueNode<E>>;
type L<E>   = List<E>;

fn empty<E: Clone> ()                           -> N<E> { Arc::new(Empty) }
fn node<E: Clone>  (back: L<E>, front: L<E>)    -> N<E> {
    match (back.len(), front.len()) {
        (0, 0) => empty(),
        _ => Arc::new(Node{ back: back.clone(), front: front.clone() })
    }
}

fn enqueue<E: Clone>(q: &N<E>, e: E) -> N<E> {
    match q.as_ref() {
        Empty                       => node(L::empty().push(e), L::empty()),
        Node { back: b, front: f }  => node(b.push(e), f.clone()),
    }
}

fn dequeue<E: Clone>(q: &N<E>) -> (E, N<E>) {
    match q.as_ref() {
        Empty                       => panic!("queue is empty"),
        Node { back: b, front: f }  =>
            match (b.len(), f.len()) {
                (0, 0) => panic!("queue is empty"),
                (_, 0) => {
                    let l = b.rev();
                    let p = l.pop();
                    (l.top().clone(), node(L::empty(), p))
                },
                (_, _) => {
                    let p = f.pop();
                    (f.top().clone(), node(b.clone(), p))
                }
            },
    }
}

fn len<E: Clone>(q: &N<E>) -> usize {
    match q.as_ref() {
        Empty => 0,
        Node { back: b, front: f } => b.len() + f.len()
    }
}

fn to_vec<E: Clone>(l: &N<E>) -> Vec<E> {
    let mut v = Vec::new();
    let mut n = l.clone();
    loop {
        match n.as_ref() {
            Empty => return v,
            _ => {
                let (e, nn) = dequeue(&n);
                v.push(e.clone());
                n = nn;
            },
        }
    }
}

#[derive(Clone)]
pub struct Queue<E: Clone> {
    n   : N<E>
}

impl<E: Clone> Queue<E> {
    ///
    /// create and return a new empty queue
    ///
    pub fn empty()                  -> Self         { Self { n: empty() } }


    ///
    /// create and return a new queue with the new element at the end
    ///
    pub fn enqueue(&self, e: E)     -> Self         { Self { n: enqueue(&self.n, e) } }

    ///
    /// create a new queue with the oldest element removed and returned
    ///
    pub fn dequeue(&self)           -> (E, Self)    {
        let (e, n) = dequeue(&self.n);
        (e, Self { n })
    }

    ///
    /// return the length of the current queue
    ///
    pub fn len(&self)               -> usize        { len(&self.n) }

    ///
    /// walk the queue and build a vector and return it (oldest elements first)
    ///
    pub fn to_vec(&self)            -> Vec<E>       { to_vec(&self.n) }
}

#[cfg(test)]
mod tests {
    use crate::queue::*;

    static mut SEED: i64 = 777;

    fn rand() -> i32 {
        unsafe {
            SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
            (SEED >> 24) as i32
        }
    }

    #[test]
    fn enqueue() {
        let mut elements    = Vec::new();
        let mut l = Queue::empty();
        for _ in 0..1000 {
            let e = rand();
            elements.push(e);
            l = l.enqueue(e);
        }

        assert_eq!(elements.len(), 1000);
        assert_eq!(elements.len(), l.len());

        let queue_elems = l.to_vec();
        for i in 0..1000 {
            assert_eq!(queue_elems[i], elements[i]);
        }
    }

    #[test]
    fn dequeue() {
        let mut elements    = Vec::new();
        let mut l = Queue::empty();
        for _ in 0..100000 {
            let e = rand();
            elements.push(e);
            l = l.enqueue(e);
        }


        assert_eq!(elements.len(), 100000);
        assert_eq!(elements.len(), l.len());

        let list_elems = l.to_vec();
        for i in 0..100000 {
            assert_eq!(list_elems[i], elements[i]);
        }

        for i in 0..50000 {
            let (e, n) = l.dequeue();
            let e2 = elements[i];
            assert_eq!(e, e2);
            l = n;
        }

        assert_eq!(l.len(), 50000);

        let queue_elems = l.to_vec();
        for i in 0..50000 {
            assert_eq!(queue_elems[i], elements[i + 50000]);
        }
    }
}