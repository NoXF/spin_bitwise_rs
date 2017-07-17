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
    //    println!("{} started", thread_idx);
    
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
//            println!("{} READ {}", thread_idx, x);
            Some(y)
        } else {
            None
        }
    }).collect();
    
    //    println!("{} Total reads: {}", thread_idx, read_locks.len());
    
    let write_locks: Vec<&RwLock<u64>> = [match locks.get(&write_key) {
        None => panic!("Could not find write key `{}`", write_key),
        Some(y) => y
    }].to_vec();
    
    for _ in 0..black_box(iter_count) {
        //        let ccc = idx as u64;
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
    
    //    locks.insert(123, RwLock::new(34))
    //;
//    println!("Total locks {}", total_locks);
    
    let thread_locks: Vec<JoinHandle<_>> = (0..total_locks).map(
        |thread_idx| {
            let locks = locks.clone();
            let start_barrier = start_barrier.clone();
//            println!("Starting {}", thread_idx);
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
    
    //    println!("Waiting {}", thread_locks.len());
    
    //    let time_begin = Instant::now();
    
    //    let thread_results: Vec<f64> = thread_locks.into_iter().map(|d| DurationWrapper::from(d.join().expect("ASD")).into()).collect();
    
    //    let writer_max_runtime: f64 = thread_results.into_iter().fold(f64::NAN, f64::max);
    
    //    let time_stopped = time_begin.elapsed();
    //
    //    let dur: f64 = DurationWrapper::from(time_stopped).into();
    
    #[allow(unused_must_use)]
    for lock in thread_locks {
        lock.join();
    }
    
    //    println!("Dumping locks: {:.2}s {:.2}s", dur, writer_max_runtime);
    
    let result = (0..total_locks).map(|x| (*(locks.get(&x).unwrap().read(0))).clone()).max().unwrap();
    let total_iter_count = (total_locks * iter_count) as u64;
    
    //    let per_op = writer_max_runtime / (total_iter_count as f64);
    //    println!("{}", per_op);
    
    assert!(result == total_iter_count, format!("{} != {}", result, total_iter_count));
    //    assert!(dur < min_duration, format!("{} >= {}", dur, min_duration));
    //    return per_op;
}


fn bench_many(b: &mut Bencher, iter_count: u32, threads: u32) {
    b.iter(move || {
        test(iter_count, threads);
    });
    
    b.bytes = threads as u64 * iter_count as u64 * 1000 * 1000;
}

#[bench]
fn bench_many_10000_2_threads(b: &mut Bencher) {
    bench_many(b, 1000000, 2);
}

#[bench]
fn bench_many_10000_3_threads(b: &mut Bencher) {
    bench_many(b, 1000000, 3);
}

#[bench]
fn bench_many_1000_4_threads(b: &mut Bencher) {
    bench_many(b, 1000000, 4);
}

#[bench]
fn bench_many_1000_5_threads(b: &mut Bencher) {
    bench_many(b, 1000000, 5);
}