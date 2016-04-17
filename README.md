rust-groupby
============

GroupBy iterator implemented without use of RefCell.

Usage:

```rust
use groupby::GroupByIterator;

for (key, grp) in vec![1,1,1,1,2,3,3,4].into_iter().groupby(|x| x/2).by_ref() {
    println!("Key {:?}", key);
    for item in grp.take(2) {
        println!(" - {:?}", item);
    }
}
```
