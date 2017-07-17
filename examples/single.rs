extern crate spin_bitwise;

use spin_bitwise::{RwLock, random_reader_idx, ARCH};

fn main() {
    let lock = RwLock::new(0);
    
    // You may either generate a random reader id
    let reader_id = random_reader_idx();
    // Or you may supply a reader id from you own threading environment
    // But it must be less than `spin_bitwise::ARCH.reader_cnt`
    let reader_id = 0 % ARCH.reader_cnt;
    
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