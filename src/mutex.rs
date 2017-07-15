use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use rand::random;
use util::cpu_relax;
use core::ops::{Drop, Deref, DerefMut};

//use std::ops::Shl;
//use std::ops::BitAnd;
//use std::ops::AddAssign;

//type LockType = usize;
const READERS_MAX: usize = 15;
const WRITER_IDX: usize = READERS_MAX;
//static MAX_SIZE: usize = 32;
//const MAX_TRY: u32 = 50000;

pub struct Mutex<T: ? Sized>
{
    lock: Arc<AtomicUsize>,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ? Sized + 'a>
{
    lock: &'a Arc<AtomicUsize>,
    data: &'a mut T,
    idx: usize,
//    mutex: &'a Mutex<T>
}

unsafe impl<T: ? Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ? Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T>
{
//    #[cfg(feature = "const_fn")]
    pub fn new(lock: Arc<AtomicUsize>, user_data: T) -> Mutex<T>
    {
        Mutex {
            lock: lock,
            data: UnsafeCell::new(user_data),
        }
    }
}

fn atomic_wait(lock: &Arc<AtomicUsize>, idx: usize) -> usize {
    lock.fetch_or(bitmask_wait(idx), Ordering::SeqCst)
}

const fn bitmask_wait(id: usize) -> usize {
    1 << (id * 2)
}

const fn bitmask_lock(id: usize) -> usize {
    1 << (id * 2 + 1)
}

const fn bitmask_both(id: usize) -> usize {
    bitmask_lock(id) | bitmask_wait(id)
}

impl<T: ? Sized> Mutex<T>
{
    fn get(&self) -> usize {
        self.lock.load(Ordering::SeqCst)
    }
    
    fn set_wait(&self, idx: usize) -> usize {
        atomic_wait(&self.lock, idx)
//        self.lock.fetch_or(bitmask_wait(idx), Ordering::SeqCst)
    }
    
    fn set_lock(&self, idx: usize) -> usize {
        self.lock.fetch_or(bitmask_lock(idx) | bitmask_wait(idx), Ordering::SeqCst)
    }
    
//    fn set_unlock(&self, idx: usize) -> usize {
//        self.lock.fetch_xor(bitmask_both(idx), Ordering::SeqCst)
//    }
    
    fn writer_mask(&self) -> usize {
        return bitmask_both(WRITER_IDX);
    }
    
    pub fn obtain_reader_lock(&self) -> usize {
        let mut idx : usize = random();
        idx = idx % READERS_MAX;
        
        let mut prev_state = self.set_wait(idx);
        
        loop {
            if prev_state & bitmask_wait(idx) == 0 {
                break
            } else {
                cpu_relax();
                
                idx = random();
                idx = idx % READERS_MAX;
            }
            
            prev_state = self.set_wait(idx);
        }
        
        loop {
            if prev_state & self.writer_mask() == 0 {
                break
            } else {
                prev_state = self.get()
            }
    
            cpu_relax();
        }
        
        self.set_lock(idx);
        
        return idx;
    }
    
    pub fn obtain_writer_lock(&self) -> usize {
        loop {
            let mask = !(self.writer_mask() | 0x15555555);
            let a = self.set_wait(WRITER_IDX) & mask;
            
            // None of the replicas are LOCKED.
            
            if a & 0xFFFFFFFF == 0 {
                break;
            }
    
            cpu_relax();
        }
        loop {
            if self.set_lock(WRITER_IDX) & bitmask_lock(WRITER_IDX) == 0 {
                break;
            }
            
            cpu_relax();
        }
        
        WRITER_IDX
    }
    
//    fn data(&mut self) -> &mut T  {
//        unsafe { &mut *self.data.get() }
//    }
    
    pub fn lock_reader(&self) -> MutexGuard<T>
    {
        MutexGuard {
            idx: self.obtain_reader_lock(),
//            mutex: &mut self,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
//            id: lock_id
        }
    }
    
    pub fn lock_writer(&self) -> MutexGuard<T>
    {
        MutexGuard {
            idx: self.obtain_writer_lock(),
//            mutex: &mut self,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
    
//    pub fn unlock(&mut self, id: usize) {
//
//    }
}

impl<'a, T: ? Sized> Deref for MutexGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ? Sized> DerefMut for MutexGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut *self.data
    }
}

impl<'a, T: ? Sized> Drop for MutexGuard<'a, T>
{
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        self.lock.fetch_xor(bitmask_both(self.idx), Ordering::SeqCst);
//        self.mutex.set_unlock(self.idx);
    }
}
