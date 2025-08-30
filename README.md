# Purely Functional Data Structures in Rust

[![Crates.io](https://img.shields.io/crates/v/pfds.svg)](https://crates.io/crates/pfds)
[![Crates.io Downloads](https://img.shields.io/crates/d/pfds)](https://crates.io/crates/pfds)
[![License](https://img.shields.io/crates/l/pfds.svg)](https://github.com/NeoCogi/pfds/blob/master/LICENSE)
[![Documentation](https://docs.rs/pfds/badge.svg)](https://docs.rs/pfds)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/NeoCogi/pfds)

## Overview

A collection of persistent (immutable) data structures implemented in Rust. These data structures use structural sharing to efficiently create new versions without modifying the original, making them ideal for functional programming, concurrent systems, and applications requiring data versioning.

## Features

- **Persistent**: All operations return new versions while preserving the original
- **Thread-safe**: Immutable by design, safe to share between threads
- **Efficient**: Uses structural sharing to minimize memory usage and copying
- **Comprehensive**: Includes lists, queues, sets, maps, and trees

## Data Structures

### Implemented

| Structure | Description | Performance |
|-----------|-------------|-------------|
| **List** | Singly-linked stack/list | push/pop/top: O(1), len: O(1) |
| **Queue** | FIFO queue with two lists | enqueue: O(1), dequeue: O(1) amortized |
| **Set** | Ordered set (AVL tree) | insert/remove/find: O(log n) |
| **Map** | Ordered key-value map (AVL tree) | insert/remove/find: O(log n) |
| **HashSet** | Hash-based set (HAMT) | insert/remove/find: O(1) average |
| **HashMap** | Hash-based key-value map (HAMT) | insert/remove/find: O(1) average |
| **Tree** | Multi-way tree with paths | Varies by operation |

### All structures now support iterators!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pfds = "0.6.0"
```

## Quick Start

```rust
use pfds::{List, Queue, Set, Map, HashSet, HashMap};

// List - Stack operations
let list = List::empty()
    .push(1)
    .push(2)
    .push(3);
assert_eq!(*list.top(), 3);
assert_eq!(list.len(), 3);

// Queue - FIFO operations  
let queue = Queue::empty()
    .enqueue(1)
    .enqueue(2);
let (first, rest) = queue.dequeue();
assert_eq!(first, 1);

// Set - Ordered unique elements
let set = Set::empty()
    .insert(5)
    .insert(3)
    .insert(7);
assert!(set.exist(3));
assert_eq!(set.to_vec(), vec![3, 5, 7]); // Sorted

// Map - Ordered key-value pairs
let map = Map::empty()
    .insert("alice", 30)
    .insert("bob", 25);
assert_eq!(map.find("alice"), Some(&30));

// HashSet - Fast unique elements
let hashset = HashSet::empty()
    .insert(100)
    .insert(200);
assert!(hashset.exist(100));

// HashMap - Fast key-value pairs
let hashmap = HashMap::empty()
    .insert("key", "value");
assert_eq!(hashmap.find("key"), Some(&"value"));
```

## API Documentation

### Common Patterns

All data structures follow similar patterns:

- `empty()` - Create a new empty structure
- `len()` / `is_empty()` - Query size
- `insert()` / `push()` / `enqueue()` - Add elements
- `remove()` / `pop()` / `dequeue()` - Remove elements
- `find()` / `exist()` / `top()` - Access elements
- `to_vec()` - Convert to vector
- `iter()` - Get an iterator

### List<T>

A persistent stack/list with O(1) push and pop operations.

```rust
use pfds::List;

let list = List::empty()
    .push(1)
    .push(2)
    .push(3);

// Access top element
assert_eq!(*list.top(), 3);

// Pop returns new list
let list2 = list.pop();
assert_eq!(list.len(), 3);  // Original unchanged
assert_eq!(list2.len(), 2);

// Iterate
for item in list.iter() {
    println!("{}", item);
}

// Reverse
let reversed = list.rev();
assert_eq!(reversed.to_vec(), vec![1, 2, 3]);
```

### Queue<T>

A persistent FIFO queue with amortized O(1) operations.

```rust
use pfds::Queue;

let queue = Queue::empty()
    .enqueue("first")
    .enqueue("second")
    .enqueue("third");

// Dequeue returns element and new queue
let (elem, rest) = queue.dequeue();
assert_eq!(elem, "first");
assert_eq!(queue.len(), 3);  // Original unchanged
assert_eq!(rest.len(), 2);

// Iterate in FIFO order
for item in queue.iter() {
    println!("{}", item);
}
```

### Set<K>

An ordered set implemented as a balanced binary tree.

```rust
use pfds::Set;

let set = Set::empty()
    .insert(5)
    .insert(3)
    .insert(7)
    .insert(3);  // Duplicate, no effect

assert_eq!(set.len(), 3);
assert!(set.exist(3));
assert!(!set.exist(4));

// Elements are kept sorted
assert_eq!(set.to_vec(), vec![3, 5, 7]);

// Remove element
let set2 = set.remove(5);
assert!(set.exist(5));   // Original unchanged
assert!(!set2.exist(5));
```

### Map<K, V>

An ordered map implemented as a balanced binary tree.

```rust
use pfds::Map;

let map = Map::empty()
    .insert("rust", 2010)
    .insert("c", 1972)
    .insert("python", 1991);

// Find returns Option<&V>
assert_eq!(map.find("rust"), Some(&2010));
assert_eq!(map.find("java"), None);

// Check existence
assert!(map.exist("c"));

// Iterate in sorted order by key
for (key, value) in map.iter() {
    println!("{}: {}", key, value);
}
```

### HashSet<K>

A hash-based set using Hash Array Mapped Trie (HAMT).

```rust
use pfds::HashSet;

let set = HashSet::empty()
    .insert(100)
    .insert(200)
    .insert(300);

assert!(set.exist(200));
assert_eq!(set.len(), 3);

// Convert to vector (unordered)
let vec = set.to_vec();
assert_eq!(vec.len(), 3);
```

### HashMap<K, V>

A hash-based map using Hash Array Mapped Trie (HAMT).

```rust
use pfds::HashMap;

let map = HashMap::empty()
    .insert("key1", "value1")
    .insert("key2", "value2");

assert_eq!(map.find("key1"), Some(&"value1"));
assert_eq!(map.len(), 2);

// Remove key
let map2 = map.remove("key1");
assert!(map.exist("key1"));   // Original unchanged
assert!(!map2.exist("key1"));
```

### Tree (Path<D>)

A persistent multi-way tree with path-based navigation.

```rust
use pfds::Path;

// Create a tree with root
let tree = Path::new("root");

// Add children
let child1 = tree.add_node("child1");
let child2 = tree.add_node("child2");

// Navigate the tree
let parent = child1.parent();
assert_eq!(*parent.data(), "root");

// Get all children of root
let children = tree.children();
assert_eq!(children.len(), 2);

// Transform entire subtree
let numbers = Path::new(1)
    .add_node(2).parent()
    .add_node(3).parent();
let doubled = numbers.apply_recursive(|x| Some(x * 2));

// Filter tree
let filtered = tree.filter_recursive(|data| data.contains("child"));

// Flatten to vector of all nodes
let all_nodes = tree.flatten();
```

## Iterators

All data structures now support iteration:

```rust
use pfds::{List, Queue, Set, Map};

// List - iterates from top to bottom
let list = List::empty().push(1).push(2).push(3);
let vec: Vec<_> = list.iter().collect();
assert_eq!(vec, vec![3, 2, 1]);

// Queue - iterates in FIFO order
let queue = Queue::empty().enqueue(1).enqueue(2).enqueue(3);
let vec: Vec<_> = queue.iter().collect();
assert_eq!(vec, vec![1, 2, 3]);

// Set - iterates in sorted order
let set = Set::empty().insert(5).insert(3).insert(7);
let vec: Vec<_> = set.iter().collect();
assert_eq!(vec, vec![3, 5, 7]);

// Map - iterates in sorted order by key
let map = Map::empty().insert(2, "b").insert(1, "a");
let vec: Vec<_> = map.iter().collect();
assert_eq!(vec, vec![(1, "a"), (2, "b")]);
```

## Performance Characteristics

| Operation | List | Queue | Set/Map | HashSet/HashMap |
|-----------|------|-------|---------|-----------------|
| Insert/Push | O(1) | O(1) | O(log n) | O(1) average |
| Remove/Pop | O(1) | O(1)* | O(log n) | O(1) average |
| Find/Exist | O(n) | O(n) | O(log n) | O(1) average |
| Length | O(1) | O(1) | O(1) | O(1) |
| To Vector | O(n) | O(n) | O(n) | O(n) |
| Iterate | O(n) | O(n) | O(n) | O(n) |

*Queue dequeue is O(1) amortized, O(n) worst case when the front list is empty

## Use Cases

Persistent data structures are ideal for:

- **Functional Programming**: Immutable by default, composable operations
- **Concurrent Systems**: No locks needed, thread-safe by design
- **Undo/Redo**: Keep history of changes efficiently
- **Backtracking Algorithms**: Explore multiple paths without copying
- **Database Systems**: Multi-version concurrency control (MVCC)
- **State Management**: Redux-like patterns, event sourcing

## Implementation Details

- **List**: Singly-linked list with size caching
- **Queue**: Two-list implementation (Okasaki's queue)
- **Set/Map**: AVL trees with height-based balancing
- **HashSet/HashMap**: Hash Array Mapped Trie with 4-bit branching
- **Tree**: Multi-way tree with path-based navigation

All structures use `Arc` for shared ownership and structural sharing.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Testing

The library aims for comprehensive test coverage:

```bash
cargo test
```

Generate documentation:

```bash
cargo doc --open
```

## Credits

The Set and Map implementations are inspired by the F# compiler code (FSharp.Core/set.fs), known for high performance.

## License

BSD-3-Clause License

## Recent Changes

### Version 0.6.0
- Added iterators for Queue, Map, and Set
- Fixed HashSet inefficiency in insert operation
- Fixed unreachable code in Queue dequeue
- Improved documentation with examples

### Bug Fixes
- Fixed incorrect panic message in List::top()
- Optimized HashSet::insert to avoid unnecessary remove/reinsert
- Changed unreachable panic to unreachable!() in Queue::dequeue