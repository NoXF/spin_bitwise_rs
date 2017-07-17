use std::sync::atomic::{AtomicUsize, Ordering};
//use std::thread::{current, ThreadId};

use rand::random;
use arch::ARCH;

type Lock<'a> = &'a AtomicUsize;

const ATOMICITY: Ordering = Ordering::SeqCst;

pub const fn bitmask_lock(id: usize) -> usize {
    1 << id
}

pub const fn bitmask_readers_lock() -> usize {
    ARCH.reader_lock_mask
}

pub fn atomic_lock(lock: Lock, idx: usize) -> usize {
    let mask = bitmask_lock(idx);
    lock.fetch_or(mask, ATOMICITY)
}

pub fn atomic_unlock(lock: Lock, idx: usize) -> usize {
    let mask = bitmask_lock(idx);
    let ret = lock.fetch_xor(mask, ATOMICITY);
    assert!(ret & mask == mask, "Can not allow to unlock a previously unlocked value");
    ret
}


pub fn random_reader_idx() -> usize {
    let r: usize = random();
    r % ARCH.reader_cnt
}

/// Returns true if we should retry the call
pub fn atomic_reader_lock(lock: Lock, idx: usize) -> (usize, bool, bool) {
    let prev_state = atomic_lock(lock, idx);
    let owned = prev_state & bitmask_lock(idx) == 0;
    
    if prev_state & bitmask_lock(ARCH.reader_cnt) == 0 {
        (prev_state, owned, false)
    } else {
        (prev_state, owned, true)
    }
}

pub fn atomic_reader_unlock(lock: Lock, idx: usize) -> (usize, bool) {
    let prev_state = lock.fetch_xor(bitmask_lock(idx), ATOMICITY);
    
    assert!(prev_state & bitmask_lock(idx) == bitmask_lock(idx), "Must not happen");
    
    (prev_state, false)
}

pub fn atomic_writer_lock(lock: Lock) -> (usize, bool, bool) {
    let prev_state = atomic_lock(lock, ARCH.reader_cnt);
    let owned = prev_state & bitmask_lock(ARCH.reader_cnt) == 0;
    
    if prev_state & bitmask_readers_lock() == 0 {
        (prev_state, owned, false)
    } else {
        (prev_state, owned, true)
    }
}

pub fn atomic_writer_unlock(lock: Lock) -> (usize, bool) {
    atomic_reader_unlock(lock, ARCH.reader_cnt)
}


