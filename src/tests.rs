#![cfg(test)]

use std::prelude::v1::*;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use super::Mutex;
use std::thread::{spawn, JoinHandle, sleep};
use std::time::Duration;


fn test_multithreaded(iter_count: u64, sleep_time: u64, log_each: u64, readers: u64, writer_pairs: u64) -> (i64, f64) {
    let spinlock = Arc::new(AtomicUsize::new(0));
    let mut variable: i64 = 0;
    let mutex = Arc::new(Mutex::new(spinlock.clone(), variable));
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    
    let ten_millis = Duration::from_millis(sleep_time);
    
    for idx_loader in 0..readers {
        let my_mutex = mutex.clone();
        
        let thread_loader = spawn(move || {
            sleep(ten_millis);
            
            for idx in 0..iter_count {
                let locked = my_mutex.lock_reader();
                
                if idx == 1 {
                    //                    println!("{} Loader: {}", idx_loader, *locked);
                }
                
                if idx % log_each == 0 {
//                    println!("{} Loader: {}", idx_loader, *locked);
                }
            }
            let locked = my_mutex.lock_reader();
            
//            println!("{} Loaded: {}", idx_loader, *locked);
        });
        
        threads.push(thread_loader);
    }
    
    for idx_writer in 0..(writer_pairs * 2) {
        let my_mutex = mutex.clone();
        
        let thread_writer = spawn(move || {
            sleep(ten_millis);
            for idx in 0..iter_count {
                let mut locked = my_mutex.lock_writer();
                
                if idx == 1 {
                    // println!("{} Writer: {}", idx_writer, *locked);
                }
                
                if idx % log_each == 0 {
//                    println!("{} Writer: {}", idx_writer, *locked);
                }
                
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
            Ok(_) => {},
            _ => panic!("Cound not help myself")
        }
    }
    
    let elapsed = now.elapsed();
    let sec: f64 = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    let ops_per_sec: f64 = ((writer_pairs * 2 * iter_count) as f64) / sec;
//    println!("Seconds: {} OpsPerSecond={}", sec, ops_per_sec);
    
    let locked = mutex.lock_reader();
//    println!("Written: {}", *locked);
    
    let locked_value = *locked;
    
    return (locked_value, ops_per_sec)
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
    let (counter, ops_per_sec) = test_multithreaded(1000000, 100, 500000, 0, 1);
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