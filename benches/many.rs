#![feature(test)]

extern crate test;
extern crate rand;
extern crate spin_bitwise;

use std::sync::{Arc, Barrier};
use std::collections::HashMap;
use spin_bitwise::RwLock;
use test::{Bencher, black_box};
use std::thread::{spawn, JoinHandle};

fn thread_locks(thread_idx: u32, total_locks: u32, overlap: u32, locks: Arc<HashMap<u32, RwLock<u64>>>, iter_count: u32) {
    let write_key = thread_idx % total_locks;
    
    let in_range = |x| {
        if thread_idx + overlap >= total_locks {
            x > thread_idx || x < (thread_idx + overlap) % total_locks
        } else {
            x > thread_idx && x < thread_idx + overlap
        }
    };
    
    let read_locks: Vec<&RwLock<u64>> = (0..total_locks).filter_map(|x| match locks.get(&x) {
        None => panic!("Could not find key `{}`", x),
        Some(y) => if x != write_key && in_range(x) {
            Some(y)
        } else {
            None
        }
    }).collect();
    
    let write_locks: Vec<&RwLock<u64>> = [match locks.get(&write_key) {
        None => panic!("Could not find write key `{}`", write_key),
        Some(y) => y
    }].to_vec();
    
    for _ in 0..black_box(iter_count) {
        let locking = RwLock::lock_many(thread_idx as usize, &read_locks, &write_locks);
        
        let checksum = locking.read.iter().map(|x| (*x).clone()).fold(0, |a, b| {
            if a > b {
                a
            } else {
                b
            }
        });
        
        for mut w in locking.write {
            *w = {
                if *w > checksum {
                    *w + 1
                } else {
                    checksum + 1
                }
            }
        }
    }
}

fn test(iter_count: u32, threads: u32) {
    let total_locks = threads;
    let overlap = threads;
    let lock_init_val: u64 = 0;
    let mut locks = HashMap::<u32, RwLock<u64>>::new();
    let start_barrier = Arc::new(Barrier::new(total_locks as usize));
    
    for idx in 0..total_locks {
        locks.insert(idx as u32, RwLock::new(lock_init_val));
    }
    
    let locks = Arc::new(locks);
    
    let thread_locks: Vec<JoinHandle<_>> = (0..total_locks).map(
        |thread_idx| {
            let locks = locks.clone();
            let start_barrier = start_barrier.clone();
            spawn(move || {
                start_barrier.wait();
                thread_locks(
                    thread_idx as u32,
                    total_locks,
                    overlap,
                    locks,
                    iter_count
                )
            })
        }
    ).collect();
    
    #[allow(unused_must_use)]
    for lock in thread_locks {
        lock.join();
    }
    
    let result = (0..total_locks).map(|x| (*(locks.get(&x).unwrap().read(0))).clone()).max().unwrap();
    let total_iter_count = (total_locks * iter_count) as u64;
    
    assert!(result == total_iter_count, format!("{} != {}", result, total_iter_count));
}

const ITER : u32 = 20000;


fn bench_many(b: &mut Bencher, iter_count: u32, threads: u32) {
    b.iter(move || {
        test(iter_count, threads);
    });
    
    b.bytes = threads as u64 * iter_count as u64 * 1000 * 1000;
}

#[bench]
fn bench_many_10000_2_threads(b: &mut Bencher) {
    bench_many(b, ITER, 2);
}

#[bench]
fn bench_many_10000_3_threads(b: &mut Bencher) {
    bench_many(b, ITER, 3);
}

#[bench]
fn bench_many_1000_4_threads(b: &mut Bencher) {
    bench_many(b, ITER, 4);
}

#[bench]
fn bench_many_1000_5_threads(b: &mut Bencher) {
    bench_many(b, ITER, 5);
}