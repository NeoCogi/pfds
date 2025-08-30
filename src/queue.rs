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

use crate::list::*;

enum QueueNode<E: Clone> {
    Empty,
    Node { back: L<E>, front: L<E> },
}

use QueueNode::*;

type N<E> = Arc<QueueNode<E>>;
type L<E> = List<E>;

fn empty<E: Clone>() -> N<E> {
    Arc::new(Empty)
}
fn node<E: Clone>(back: L<E>, front: L<E>) -> N<E> {
    match (back.len(), front.len()) {
        (0, 0) => empty(),
        _ => Arc::new(Node { back, front }),
    }
}

fn enqueue<E: Clone>(q: &N<E>, e: E) -> N<E> {
    match q.as_ref() {
        Empty => node(L::empty().push(e), L::empty()),
        Node { back: b, front: f } => node(b.push(e), f.clone()),
    }
}

fn dequeue<E: Clone>(q: &N<E>) -> (E, N<E>) {
    match q.as_ref() {
        Empty => panic!("queue is empty"),
        Node { back: b, front: f } => match (b.len(), f.len()) {
            (0, 0) => unreachable!("node() invariant violated: Node should never have both lists empty"),
            (_, 0) => {
                let l = b.rev();
                let p = l.pop();
                (l.top().clone(), node(L::empty(), p))
            }
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
        Node { back: b, front: f } => b.len() + f.len(),
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
            }
        }
    }
}

/// A persistent (immutable) FIFO queue data structure.
/// 
/// `Queue` is implemented using two lists (front and back) to achieve
/// amortized O(1) enqueue and dequeue operations. All operations return
/// a new queue, leaving the original unchanged.
/// 
/// # Performance
/// 
/// - `enqueue`: O(1)
/// - `dequeue`: O(1) amortized, O(n) worst case when reversing back list
/// - `is_empty`: O(1)
/// - `len`: O(1) - length is cached
/// - `to_vec`: O(n)
#[derive(Clone)]
pub struct Queue<E: Clone> {
    n: N<E>,
}

impl<E: Clone> Queue<E> {
    /// Creates a new empty queue.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let queue: Queue<i32> = Queue::empty();
    /// assert!(queue.is_empty());
    /// assert_eq!(queue.len(), 0);
    /// ```
    pub fn empty() -> Self {
        Self { n: empty() }
    }

    /// Creates a new queue with the given element added to the back.
    /// 
    /// This operation is O(1) and shares structure with the original queue.
    /// 
    /// # Arguments
    /// 
    /// * `e` - The element to add to the back of the queue
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let queue = Queue::empty().enqueue(1).enqueue(2).enqueue(3);
    /// assert_eq!(queue.len(), 3);
    /// ```
    pub fn enqueue(&self, e: E) -> Self {
        Self {
            n: enqueue(&self.n, e),
        }
    }

    /// Removes and returns the front element and a new queue without that element.
    /// 
    /// This operation is O(1) amortized. When the front list is empty,
    /// it reverses the back list which takes O(n) time.
    /// 
    /// # Returns
    /// 
    /// A tuple containing:
    /// - The front element
    /// - A new queue without the front element
    /// 
    /// # Panics
    /// 
    /// Panics if the queue is empty.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let queue = Queue::empty().enqueue(1).enqueue(2);
    /// let (first, queue2) = queue.dequeue();
    /// assert_eq!(first, 1);
    /// assert_eq!(queue.len(), 2);  // Original unchanged
    /// assert_eq!(queue2.len(), 1);
    /// ```
    pub fn dequeue(&self) -> (E, Self) {
        let (e, n) = dequeue(&self.n);
        (e, Self { n })
    }

    /// Returns true if the queue is empty.
    /// 
    /// This operation is O(1).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let empty = Queue::<i32>::empty();
    /// assert!(empty.is_empty());
    /// 
    /// let non_empty = empty.enqueue(1);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        len(&self.n) == 0
    }

    /// Returns the number of elements in the queue.
    /// 
    /// This operation is O(1) as the length is cached.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let queue = Queue::empty().enqueue(1).enqueue(2).enqueue(3);
    /// assert_eq!(queue.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        len(&self.n)
    }

    /// Converts the queue to a vector.
    /// 
    /// Elements are returned in FIFO order (oldest elements first).
    /// This operation is O(n).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let queue = Queue::empty().enqueue(1).enqueue(2).enqueue(3);
    /// let vec = queue.to_vec();
    /// assert_eq!(vec, vec![1, 2, 3]); // FIFO order
    /// ```
    pub fn to_vec(&self) -> Vec<E> {
        to_vec(&self.n)
    }

    /// Returns an iterator over the queue elements.
    /// 
    /// The iterator yields elements in FIFO order (oldest first).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pfds::Queue;
    /// 
    /// let queue = Queue::empty().enqueue(1).enqueue(2).enqueue(3);
    /// let collected: Vec<_> = queue.iter().collect();
    /// assert_eq!(collected, vec![1, 2, 3]);
    /// ```
    pub fn iter(&self) -> QueueIter<E> {
        QueueIter {
            queue: self.n.clone(),
            _phantom: PhantomData::default(),
        }
    }
}

/// An iterator over the elements of a `Queue`.
/// 
/// This struct is created by the [`Queue::iter`] method.
/// The iterator yields elements in FIFO order.
pub struct QueueIter<'a, E: Clone> {
    queue: N<E>,
    _phantom: PhantomData<&'a E>,
}

impl<'a, E: Clone> std::iter::Iterator for QueueIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.as_ref() {
            Empty => None,
            _ => {
                let (elem, new_queue) = dequeue(&self.queue);
                self.queue = new_queue;
                Some(elem)
            }
        }
    }
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
        let mut elements = Vec::new();
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
        let mut elements = Vec::new();
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

    #[test]
    fn iter() {
        let mut elements = Vec::new();
        let mut q = Queue::empty();
        for _ in 0..1000 {
            let e = rand();
            elements.push(e);
            q = q.enqueue(e);
        }

        assert_eq!(elements.len(), 1000);
        assert_eq!(elements.len(), q.len());

        // Test iterator yields elements in FIFO order
        let mut count = 0;
        for elem in q.iter() {
            assert_eq!(elem, elements[count]);
            count += 1;
        }
        assert_eq!(count, 1000);

        // Test that we can iterate multiple times (persistent data structure)
        let collected: Vec<i32> = q.iter().collect();
        assert_eq!(collected, elements);
    }
}
