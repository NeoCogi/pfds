# Purely Functional Data Structures in Rust

## Rationale

Purely functional data structures have the persistence property. The data structure is a collection of delta updates on top of previous updates. They are immutable and as such, they are very suitable solution for a large set of problems in distributed systems, concurrent systems and databases, ...

## What's in there

- [x] List or Stack
- [x] Queue
- [x] Balanced Set
- [x] Balanced Map
- [x] Hash Set
- [x] Hash Map
- [x] Tree

## What's excluded

`Map`/`Set`/`Queue` `iter` are not yet implemented. They are available for `List`/`HashSet`/`HashMap`/`Tree` however. 
## Example

```rust
let mut numbers = Vec::new();
let mut n   = HashSet::empty();
for _ in 0..1000000 {
    let r = rand() % 100000;
    n   = n.insert(r);
    numbers.push(r);
}

let mut sorted  = numbers.clone();
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
```

## Test coverage

The tests aim for 100% test coverage. 100% coverage doesn't exclude bugs. In fact it uncovered bugs in the coverage tool (tarpaulin), so use it at your own risk ;)
Also, given the fragile status of tarpaulin, there are a lot of false positive: code marked as uncovered, but it is.

## Credits

Bot set.rs & map.rs are highly inspired by the F# code in the F# compiler code (FSharp.Core/set.fs). The F# code is one of the highest performance implementations out there.

## License
BSD-3-Clause license
