use std::sync::atomic::{AtomicUsize, Ordering};
//use std::thread::{current, ThreadId};

use rand::random;
use arch::ARCH;

type Lock<'a> = &'a AtomicUsize;

pub const ATOMICITY_LOAD: Ordering = Ordering::Relaxed;
pub const ATOMICITY_LOCK: Ordering = Ordering::Acquire;
pub const ATOMICITY_RELEASE: Ordering = Ordering::Release;

#[inline(always)]
pub const fn bitmask_lock(id: usize) -> usize {
    1 << id
}

#[inline(always)]
pub const fn bitmask_readers_lock() -> usize {
    ARCH.reader_lock_mask
}

#[inline(always)]
pub fn atomic_load(lock: Lock) -> usize {
    lock.load(ATOMICITY_LOAD)
}

#[inline(always)]
pub fn atomic_lock(lock: Lock, idx: usize) -> usize {
    let mask = bitmask_lock(idx);
    lock.fetch_or(mask, ATOMICITY_LOCK)
}

#[inline(always)]
pub fn atomic_unlock(lock: Lock, idx: usize) -> usize {
    let mask = bitmask_lock(idx);
    let ret = lock.fetch_xor(mask, ATOMICITY_RELEASE);
    //    assert!(ret & mask == mask, "Can not allow to unlock a previously unlocked value");
    ret
}


pub fn random_reader_idx() -> usize {
    let r: usize = random();
    r % ARCH.reader_cnt
}

/// Returns true if we should retry the call
#[inline(always)]
pub fn atomic_reader_lock(lock: Lock, idx: usize) -> (usize, bool, bool) {
    let prev_state = atomic_lock(lock, idx);
    let owned = prev_state & bitmask_lock(idx) == 0;
    let block = prev_state & bitmask_lock(ARCH.reader_cnt) != 0;
    
    (prev_state, owned, block)
}

#[inline(always)]
pub fn atomic_reader_unlock(lock: Lock, idx: usize) -> (usize, bool) {
    let prev_state = atomic_unlock(lock, idx);
    
    //    assert!(prev_state & bitmask_lock(idx) == bitmask_lock(idx), "Must not happen");
    
    (prev_state, false)
}

#[inline(always)]
pub fn atomic_readers_free(lock: Lock) -> bool {
    atomic_load(lock) & bitmask_readers_lock() == 0
}

#[inline(always)]
pub fn atomic_reader_load(lock: Lock, idx: usize) -> bool {
    atomic_load(lock) & bitmask_lock(idx) == 0
}

#[inline(always)]
pub fn atomic_writer_free(lock: Lock) -> bool {
    atomic_load(lock) & bitmask_lock(ARCH.reader_cnt) == 0
}

#[inline(always)]
pub fn atomic_writer_lock(lock: Lock) -> (usize, bool, bool) {
    let prev_state = lock.fetch_or(bitmask_lock(ARCH.reader_cnt) | ARCH.reader_lock_mask, ATOMICITY_LOCK);
    let owned = prev_state & bitmask_lock(ARCH.reader_cnt) == 0;
    let block = prev_state & ARCH.reader_lock_mask != 0;
    
    // returns readers which must be later unlocked
    ((!prev_state) & ARCH.reader_lock_mask, owned, block)
}

#[inline(always)]
pub fn atomic_writer_unlock(lock: Lock) -> (usize, bool) {
    atomic_reader_unlock(lock, ARCH.reader_cnt)
}


