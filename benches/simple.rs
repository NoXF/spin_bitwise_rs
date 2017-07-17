#![feature(test)]

extern crate test;
extern crate spin_bitwise;

use test::{Bencher, black_box};

use spin_bitwise::RwLock;

#[bench]
fn bench_reads(b: &mut Bencher) {
    let total_iter = 1000000;
    let lock = RwLock::new(0);
    
    b.iter(|| {
        for i in 0..black_box(total_iter) {
            let locked = lock.read(0);
            *locked;
        }
    });
    
    b.bytes = total_iter * 1000 * 1000;
}

#[bench]
fn bench_writes(b: &mut Bencher) {
    let total_iter = 1000000;
    let lock = RwLock::new(0);
    
    b.iter(|| {
        for i in 0..black_box(total_iter) {
            let mut locked = lock.write();
            *locked += 1;
        }
    });
    
    b.bytes = total_iter * 1000 * 1000;
}