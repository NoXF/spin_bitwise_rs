use std::sync::atomic::{AtomicUsize, Ordering};
//use std::thread::{current, ThreadId};

use rand::random;
use arch::ARCH;

type Lock<'a> = &'a AtomicUsize;

const ATOMICITY: Ordering = Ordering::SeqCst;

//pub fn atomic_load(lock: Lock) -> usize {
//    lock.load(Ordering::Acquire)
//}

pub fn atomic_lease(lock: Lock, idx: usize) -> usize {
    lock.fetch_or(bitmask_lease(idx), ATOMICITY)
}

pub fn atomic_unlease(lock: Lock, idx: usize) -> usize {
    let ret = lock.fetch_xor(bitmask_lease(idx), ATOMICITY);
    assert!(ret & bitmask_lease(idx) == bitmask_lease(idx), "Can not allow to unlock a previously unlocked value");
    ret
}

pub fn atomic_lock(lock: Lock, idx: usize) -> usize {
    lock.fetch_or(bitmask_lock(idx) | bitmask_lease(idx), ATOMICITY)
}

pub fn atomic_unlock(lock: Lock, idx: usize) -> usize {
    let ret = lock.fetch_xor(bitmask_both(idx), ATOMICITY);
    assert!(ret & bitmask_both(idx) == bitmask_both(idx), "Can not allow to unlock a previously unlocked value");
    ret
}

pub const fn bitmask_lease(id: usize) -> usize {
    1 << (id * 2)
}

pub const fn bitmask_lock(id: usize) -> usize {
    1 << (id * 2 + 1)
}

pub const fn bitmask_both(id: usize) -> usize {
    bitmask_lock(id) | bitmask_lease(id)
}

pub const fn bitmask_writer() -> usize {
    bitmask_both(ARCH.reader_cnt)
}

pub const fn bitmask_readers_lock() -> usize {
    !(bitmask_writer() | ARCH.reader_lease_mask)
}

//pub fn random_reader_idx() -> usize {
//    random
//    (current().id().0 as usize) % ARCH.reader_cnt
//}

pub fn random_reader_idx() -> usize {
    let r: usize = random();
    r % ARCH.reader_cnt
}


/// Returns true if we should retry the call
/// May force inlining
pub fn atomic_reader_lease(lock: Lock, idx: usize) -> (usize, bool) {
    let prev_state = atomic_lease(lock, idx);
    
    if prev_state & bitmask_lease(idx) == 0 {
        (prev_state, false)
    } else {
        (prev_state, true)
    }
}

pub fn atomic_reader_unlease(lock: Lock, idx: usize) -> (usize, bool) {
    let prev_state = atomic_unlease(lock, idx);
    
    assert!(prev_state & bitmask_lease(idx) == bitmask_lease(idx), "Must not happen");
    
    (prev_state, false)
}

/// Returns true if we should retry the call
pub fn atomic_reader_lock(lock: Lock, idx: usize) -> (usize, bool) {
    let prev_state = atomic_lock(lock, idx);
    
    if prev_state & bitmask_lock(ARCH.reader_cnt) == 0 {
        (prev_state, false)
    } else {
        (prev_state, true)
    }
}

pub fn atomic_reader_unlock(lock: Lock, idx: usize) -> (usize, bool) {
    let prev_state = lock.fetch_xor(bitmask_lock(idx), ATOMICITY);
    
    assert!(prev_state & bitmask_lock(idx) == bitmask_lock(idx), "Must not happen");
    
    (prev_state, false)
}

pub fn atomic_writer_lease(lock: Lock) -> (usize, bool) {
    atomic_reader_lease(lock, ARCH.reader_cnt)
}

pub fn atomic_writer_unlease(lock: Lock) -> (usize, bool) {
    atomic_reader_unlease(lock, ARCH.reader_cnt)
}

pub fn atomic_writer_lock(lock: Lock) -> (usize, bool) {
    let prev_state = atomic_lock(lock, ARCH.reader_cnt);
    if prev_state & bitmask_readers_lock() == 0 {
        (prev_state, false)
    } else {
        (prev_state, true)
    }
}

pub fn atomic_writer_unlock(lock: Lock) -> (usize, bool) {
    atomic_reader_unlock(lock, ARCH.reader_cnt)
}


