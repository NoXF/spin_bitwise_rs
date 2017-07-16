#![cfg(test)]

use std::prelude::v1::*;

use std::sync::Arc;
use arch::ARCH;
use super::RwLock;
use core::num::dec2flt::rawfp::RawFloat;
use std::thread::{spawn, sleep, JoinHandle};
use std::time::{Duration, Instant};

#[allow(unused_variables)]
fn thread_reader(thread_idx: u64, sleep_initial: Duration, iter_count: u64, lock: Arc<RwLock<i64>>) -> Duration {
    sleep(sleep_initial);
    
    let now = Instant::now();
    
    for idx in 0..iter_count {
        let locked = lock.read((thread_idx as usize) % ARCH.reader_cnt);
    }
    let locked = lock.read((thread_idx as usize) % ARCH.reader_cnt);
    
    now.elapsed()
}

#[allow(unused_variables)]
fn thread_writer(thread_idx: u64, sleep_initial: Duration, iter_count: u64, lock: Arc<RwLock<i64>>) -> Duration {
    sleep(sleep_initial);
    
    let now = Instant::now();
    
    for idx in 0..iter_count {
        let mut locked = lock.write();
        
        if thread_idx % 2 == 0 {
            *locked += 1
        } else {
            *locked -= 1
        }
    }
    
    let locked = lock.read((thread_idx as usize) % ARCH.reader_cnt);
    
    now.elapsed()
}

struct DurationWrapper {
    duration: Duration
}

impl Into<f64> for DurationWrapper {
    fn into(self) -> f64 {
        (self.duration.as_secs() as f64) + (self.duration.subsec_nanos() as f64 / 1000_000_000.0)
    }
}

impl From<Duration> for DurationWrapper {
    fn from(d: Duration) -> Self {
        DurationWrapper { duration: d }
    }
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn test_multithreaded(iter_count: u64, sleep_time: u64, log_each: u64, readers: u64, writers: u64) -> (i64, f64) {
    let mut variable: i64 = 0;
    let lock = Arc::new(RwLock::new(variable));
    let mut durations: Vec<Duration> = Vec::new();
    
    let ten_millis = Duration::from_millis(sleep_time);
    
    let thread_reader : Vec<JoinHandle<_>> = (0..readers).map(
        |idx| {
            let my_lock = lock.clone();
            spawn(move || {
                thread_reader(idx, ten_millis, iter_count, my_lock)
            })
        }
    ).collect();
    
    let thread_writer : Vec<JoinHandle<_>> = (0..writers).map(
        |idx| {
            let my_lock = lock.clone();
            spawn(move || {
                thread_writer(idx, ten_millis, iter_count, my_lock)
            })
        }
    ).collect();
    
    let now = Instant::now();
    
    let thread_reader: Vec<f64> = thread_reader.into_iter().map(|d| DurationWrapper::from(d.join().expect("ASD")).into()).collect();
    
    let durations_writer: Vec<f64> = thread_writer.into_iter().map(|d| DurationWrapper::from(d.join().expect("ASD")).into()).collect();
    
    let writer_max_runtime: f64 = durations_writer.iter().cloned().fold(
        f64::NAN,
        f64::max
    );
    
    let reader_max_runtime: f64 = thread_reader.iter().cloned().fold(
        f64::NAN,
        f64::max
    );
    
    let wops_per_sec: f64 = (iter_count) as f64 / writer_max_runtime;
    
    let rops_per_sec: f64 = (iter_count) as f64 / reader_max_runtime;
    
    let locked = lock.read(0 as usize);
    
    let mut locked_value = *locked;
    let iter_count: i64 = iter_count as i64;
    
    let elapsed: f64 = DurationWrapper::from(now.elapsed()).into();
    
    println!(
        "I={} Readers={} Writers={} Rspd={} WSpd={}, T={}, WmT={}, RmT={}", iter_count,
        readers, writers, rops_per_sec, wops_per_sec,
        elapsed, writer_max_runtime, reader_max_runtime);
    
    println!("{:064b}", lock.state());
    
    if writers % 2 == 1 {
        locked_value -= iter_count
    }
    
    return (locked_value, wops_per_sec);
}

const MILLION: f64 = 1000. * 1000.;
const ITER: u64 = 100000;

#[test]
//#[ignore]
fn test_many() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 10, 3);
    let compare = 0.5 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
#[ignore]
fn test_simple() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 0, 2);
    let compare = 3. * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
//#[ignore]
fn test_1_readers_0_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 1, 0);
    let compare = 0.2 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
//#[ignore]
fn test_0_readers_1_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER*10, 100, 500000, 0, 1);
    let compare = 0.2 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}


#[test]
#[ignore]
fn test_64_readers_0_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 64, 0);
    let compare = 0.2 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
#[ignore]
fn test_15_readers_1_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 15, 1);
    let compare = 0.2 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
#[ignore]
fn test_1_reader_15_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 1, 15);
    let compare = 0.2 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
#[ignore]
fn test_64_reader_1_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 64, 1);
    let compare = 0.05 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
#[ignore]
fn test_64_reader_64_writer() {
    let (counter, ops_per_sec) = test_multithreaded(ITER, 100, 500000, 64, 64);
    let compare = 0.05 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}