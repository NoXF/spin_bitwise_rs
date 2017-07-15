use std::sync::atomic::{AtomicUsize, Ordering};

use rand::random;
use arch::ARCH;

pub fn atomic_wait(lock: &AtomicUsize, idx: usize) -> usize {
    lock.fetch_or(bitmask_wait(idx), Ordering::SeqCst)
}

pub fn atomic_lock(lock: &AtomicUsize, idx: usize) -> usize {
    lock.fetch_or(bitmask_lock(idx) | bitmask_wait(idx), Ordering::SeqCst)
}

pub fn atomic_unlock(lock: &AtomicUsize, idx: usize) -> usize {
    lock.fetch_xor(bitmask_both(idx), Ordering::SeqCst)
}

pub const fn bitmask_wait(id: usize) -> usize {
    1 << (id * 2)
}

pub const fn bitmask_lock(id: usize) -> usize {
    1 << (id * 2 + 1)
}

pub const fn bitmask_both(id: usize) -> usize {
    bitmask_lock(id) | bitmask_wait(id)
}

pub const fn bitmask_writer() -> usize {
    bitmask_both(ARCH.reader_cnt)
}

pub const fn bitmask_readers_lock() -> usize {
    !(bitmask_writer() | ARCH.reader_wait_mask)
}

pub fn random_reader_idx() -> usize {
    let r: usize = random();
    r % ARCH.reader_cnt
}
