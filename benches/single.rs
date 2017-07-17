#![feature(test)]
use std::prelude::v1::*;

extern crate test;
extern crate rand;
extern crate spin_bitwise;

use test::{Bencher, black_box};

use std::sync::{Arc, Barrier};
use spin_bitwise::ARCH;
use spin_bitwise::RwLock;
use std::thread::{spawn, JoinHandle};

#[allow(unused_variables)]
fn thread_reader(thread_idx: u64, iter_count: u64, lock: Arc<RwLock<i64>>) {
    for idx in 0..black_box(iter_count) {
        let locked = lock.read((thread_idx as usize) % ARCH.reader_cnt);
    }
    let locked = lock.read((thread_idx as usize) % ARCH.reader_cnt);
}

#[allow(unused_variables)]
fn thread_writer(thread_idx: u64, iter_count: u64, lock: Arc<RwLock<i64>>) {
    for idx in 0..black_box(iter_count) {
        let mut locked = lock.write();
        
        if thread_idx % 2 == 0 {
            *locked += 1
        } else {
            *locked -= 1
        }
    }
    
    let locked = lock.read((thread_idx as usize) % ARCH.reader_cnt);
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn bench_multithreaded(iter_count: u64, readers: u64, writers: u64) {
    let mut variable: i64 = 0;
    let lock = Arc::new(RwLock::new(variable));
    let start_barrier = Arc::new(Barrier::new((readers + writers) as usize));
    
    let thread_reader: Vec<JoinHandle<_>> = (0..readers).map(
        |idx| {
            let lock = lock.clone();
            let start_barrier = start_barrier.clone();
            spawn(move || {
                start_barrier.wait();
                thread_reader(idx, iter_count, lock)
            })
        }
    ).collect();
    
    let thread_writer: Vec<JoinHandle<_>> = (0..writers).map(
        |idx| {
            let lock = lock.clone();
            let start_barrier = start_barrier.clone();
            spawn(move || {
                start_barrier.wait();
                thread_writer(idx, iter_count, lock)
            })
        }
    ).collect();
    
    #[allow(unused_must_use)]
    for r in thread_reader {
        r.join();
    }
    
    #[allow(unused_must_use)]
    for w in thread_writer {
        w.join();
    }
    
    let locked = lock.read(0);
    
    let mut counter = *locked;
    
    if writers % 2 == 1 {
        counter -= iter_count as i64
    }
    
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
}

fn bench(b: &mut Bencher, iter_count: u64, readers: u64, writers: u64) {
    b.iter(|| {
        bench_multithreaded(iter_count, readers, writers)
    });
    
    b.bytes = (iter_count * readers + iter_count * writers) * 1000 * 1000;
}

const ITER : u64 = 10000;

#[bench]
fn bench_10_readers_3_writers(b: &mut Bencher) {
    bench(b, ITER, 10, 3);
}

#[bench]
fn bench_0_readers_2_writers(b: &mut Bencher) {
    bench(b, ITER, 0, 2);
}

#[bench]
fn bench_1_readers_0_writers(b: &mut Bencher) {
    bench(b, ITER, 1, 0);
}

#[bench]
fn bench_0_readers_1_writers(b: &mut Bencher) {
    bench(b, ITER, 0, 1);
}

#[bench]
fn bench_64_readers_0_writers(b: &mut Bencher) {
    bench(b, ITER, 64, 0);
}

#[bench]
fn bench_15_readers_0_writers(b: &mut Bencher) {
    bench(b, ITER, 15, 0);
}

#[bench]
fn bench_0_readers_15_writers(b: &mut Bencher) {
    bench(b, ITER, 0, 15);
}

#[bench]
fn bench_15_readers_1_writers(b: &mut Bencher) {
    bench(b, ITER, 15, 1);
}

#[bench]
fn bench_1_readers_15_writers(b: &mut Bencher) {
    bench(b, ITER, 1, 15);
}

#[bench]
fn bench_64_readers_1_writers(b: &mut Bencher) {
    bench(b, ITER, 64, 1);
}

#[bench]
fn bench_64_readers_64_writers(b: &mut Bencher) {
    bench(b, ITER, 64, 64);
}