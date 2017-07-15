#![cfg(test)]

use std::prelude::v1::*;

use std::sync::Arc;
use super::Mutex;
use std::thread::{spawn, JoinHandle, sleep};
use std::time::Duration;


#[allow(unused_variables)]
#[allow(unused_mut)]
fn test_multithreaded(iter_count: u64, sleep_time: u64, log_each: u64, readers: u64, writers: u64) -> (i64, f64) {
    let mut variable: i64 = 0;
    let mutex = Arc::new(Mutex::new(variable));
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    
    let ten_millis = Duration::from_millis(sleep_time);
    
    for idx_loader in 0..readers {
        let my_mutex = mutex.clone();
        
        let thread_loader = spawn(move || {
            sleep(ten_millis);
            
            for idx in 0..iter_count {
                let locked = my_mutex.lock_reader();
                
                if idx == 1 {}
                
                if idx % log_each == 0 {}
            }
            let locked = my_mutex.lock_reader();
        });
        
        threads.push(thread_loader);
    }
    
    for idx_writer in 0..writers {
        let my_mutex = mutex.clone();
        
        let thread_writer = spawn(move || {
            sleep(ten_millis);
            for idx in 0..iter_count {
                let mut locked = my_mutex.lock_writer();
                
                if idx == 1 {}
                
                if idx % log_each == 0 {}
                
                if idx_writer % 2 == 0 {
                    *locked += 1
                } else {
                    *locked -= 1
                }
            }
        });
        
        threads.push(thread_writer);
    }
    
    use std::time::Instant;
    let now = Instant::now();
    
    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            _ => panic!("Cound not help myself")
        }
    }
    
    let elapsed = now.elapsed();
    let sec: f64 = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    let wops_per_sec: f64 = ((writers * iter_count) as f64) / sec;
    let rops_per_sec: f64 = ((readers * iter_count) as f64) / sec;
    let locked = mutex.lock_reader();
    
    let mut locked_value = *locked;
    let iter_count : i64 = iter_count as i64;
    
    println!("I={} Readers={} Writers={} Rspd={} WSpd={}", iter_count, readers, writers, rops_per_sec, wops_per_sec);
    
    if writers % 2 == 1 {
        locked_value -= iter_count
    }
    
    return (locked_value, wops_per_sec);
}

const MILLION: f64 = 1000. * 1000.;

#[test]
//#[ignore]
fn test_many() {
    let (counter, ops_per_sec) = test_multithreaded(1000000, 100, 500000, 10, 3);
    let compare = 0.5 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
//#[ignore]
fn test_simple() {
    let (counter, ops_per_sec) = test_multithreaded(1000000, 100, 500000, 0, 2);
    let compare = 3. * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}

#[test]
fn test_15_readers_1_writer() {
    let (counter, ops_per_sec) = test_multithreaded(1000000, 100, 500000, 15, 1);
    let compare = 0.2 * MILLION;
    assert!(counter == 0, format!("At the end, we must have 0 items left in the counter (ACTUAL: {})", counter));
    assert!(ops_per_sec > compare, format!("Must be faster than {} (ACTUAL: {})", compare, ops_per_sec));
}