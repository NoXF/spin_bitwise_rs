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