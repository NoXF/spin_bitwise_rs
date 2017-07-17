spin_bitwise
===========

This Rust library implements a multiple-reader single-writer spinlock ([readers-writer lock](https://en.wikipedia.org/wiki/Readers–writer_lock)) based on a single atomic construct.

On top of this, it implements a mechanism to obtain a set of simultaneous read/write locks in a all-or-none fashion.

## TODO

 - Merge this with [spin](https://github.com/mvdnes/spin-rs)
 - Merge this with [concurrent-hashmap](https://github.com/veddan/rust-concurrent-hashmap/blob/master/benches/concurrent.rs)
 - Create a separate set of tests. Currently all of the test checks are done in benchmarks.
 - Implement docs.

## Usage
See [examples](https://github.com/andreycizov/spin_bitwise_rs/tree/master/examples), or for more thorough usage patterns see [benches](https://github.com/andreycizov/spin_bitwise_rs/tree/master/benches).

### Single example

```rust
extern crate spin_bitwise;


fn main() {
    let lock = spin_bitwise::RwLock::new(0);
    
    // You may either generate a random reader id
    let reader_id = spin_bitwise::random_reader_idx();
    // Or you may supply a reader id from you own threading environment
    // But it must be less than `spin_bitwise::ARCH.reader_cnt`
    let reader_id = 0 % spin_bitwise::ARCH.reader_cnt;
    
    {
        let mut locked = lock.write();
        *locked = 2;
    }
    {
        let mut locked = lock.write();
        *locked += 2;
    }
    
    {
        let mut locked = lock.read(reader_id);
        
        println!("Value behind the lock is: {}", *locked);
    }
}

```

Prints

```
Value behind the lock is: 4
```

### Multi-locking example

```rust
extern crate spin_bitwise;


use std::collections::HashMap;
use spin_bitwise::{RwLock, random_reader_idx, ARCH};

fn main() {
    let total_locks = 6;
    let read_count = 3;
    let write_count = 2;
    
    let mut locks = HashMap::<u32, RwLock<u64>>::new();
    
    for idx in 0..total_locks {
        locks.insert(idx as u32, RwLock::new(0));
    }
    
    // You may either generate a random reader id
    let reader_id = random_reader_idx();
    // Or you may supply a reader id from you own threading environment
    // But it must be less than `spin_bitwise::ARCH.reader_cnt`
    let reader_id = 0 % ARCH.reader_cnt;
    
    // Make sure `read_locks` and `write_locks` do not overlap
    let read_locks = (0..read_count).map(|x| locks.get(&x).unwrap()).collect();
    let write_locks = (read_count..read_count + write_count).map(|x| locks.get(&x).unwrap()).collect();
    
    {
        let locked = RwLock::lock_many(reader_id, &read_locks, &write_locks);
        
        for mut x in locked.write {
            *x += 1;
            println!("Writing lock value: {}", *x);
        }
        
        for x in locked.read {
            println!("Accessing lock value: {}", *x);
        }
    }
    
    for x in 0..total_locks {
        let locked = locks.get(&x).unwrap().read(reader_id);
        println!("Key {} Value={}", x, *locked);
    }
}
```

Prints

```
Writing lock value: 1
Writing lock value: 1
Accessing lock value: 0
Accessing lock value: 0
Accessing lock value: 0
Key 0 Value=0
Key 1 Value=0
Key 2 Value=0
Key 3 Value=1
Key 4 Value=1
Key 5 Value=0
```

## Implementation

Based on the target platform, we are using a single atomic construct to allow us to have `bit-1` read locks and a `1` write lock at the same time.

For example, for a 64-bit platform we are allowed to have `63` simultaneous readers and `1` writer.

We use an atomic xor and atomic or-get.

## Benchmarks
### Run them yourself

```bash
cargo bench
```

### Reference
MB/s here is the number of operations per second.

bench_many implements a ring of N threads each writing to a single key and reading from N-1 keys.

```bash
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/release/deps/many-a2c859d7a0e577ef

running 4 tests
test bench_many_10000_2_threads ... bench:   5,478,389 ns/iter (+/- 378,820) = 3650708 MB/s
test bench_many_10000_3_threads ... bench:  14,711,373 ns/iter (+/- 2,236,510) = 2039238 MB/s
test bench_many_1000_4_threads  ... bench:   2,930,114 ns/iter (+/- 374,163) = 1365134 MB/s
test bench_many_1000_5_threads  ... bench:   4,071,063 ns/iter (+/- 427,773) = 1228180 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 4 measured; 0 filtered out

     Running target/release/deps/simple-18571219f4ce00c8

running 2 tests
test bench_reads  ... bench:  21,121,984 ns/iter (+/- 2,327,631) = 47344037 MB/s
test bench_writes ... bench:  21,624,136 ns/iter (+/- 2,246,515) = 46244622 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 2 measured; 0 filtered out

     Running target/release/deps/single-85405c59528ad9b9

running 11 tests
test bench_0_readers_15_writers  ... bench:   5,339,556 ns/iter (+/- 1,354,088) = 2809222 MB/s
test bench_0_readers_1_writers   ... bench:      52,969 ns/iter (+/- 2,217) = 18878966 MB/s
test bench_0_readers_2_writers   ... bench:     210,795 ns/iter (+/- 18,109) = 9487891 MB/s
test bench_10_readers_3_writers  ... bench:   7,119,932 ns/iter (+/- 3,038,918) = 1825860 MB/s
test bench_15_readers_0_writers  ... bench:   2,855,392 ns/iter (+/- 1,170,957) = 5253219 MB/s
test bench_15_readers_1_writers  ... bench:   7,131,126 ns/iter (+/- 2,397,169) = 2243684 MB/s
test bench_1_readers_0_writers   ... bench:      52,919 ns/iter (+/- 2,458) = 18896804 MB/s
test bench_1_readers_15_writers  ... bench:   9,386,743 ns/iter (+/- 2,179,319) = 1704531 MB/s
test bench_64_readers_0_writers  ... bench:  16,515,874 ns/iter (+/- 2,549,464) = 3875059 MB/s
test bench_64_readers_1_writers  ... bench:  18,174,800 ns/iter (+/- 34,026,953) = 3576380 MB/s
test bench_64_readers_64_writers ... bench: 184,001,578 ns/iter (+/- 673,252,095) = 695646 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 11 measured; 0 filtered out
```

## Notes

Some of the code examples have been borrowed from (https://github.com/mvdnes/spin-rs)

