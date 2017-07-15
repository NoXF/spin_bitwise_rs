use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use rand::random;
use util::cpu_relax;
use core::ops::{Drop, Deref, DerefMut};

//use std::ops::Shl;
//use std::ops::BitAnd;
//use std::ops::AddAssign;;

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

pub struct ReadMutexGuard<'a, T: ? Sized + 'a>
{
    lock: &'a Arc<AtomicUsize>,
    data: &'a T,
    idx: usize,
}

pub struct WriteMutexGuard<'a, T: ? Sized + 'a>
{
    lock: &'a Arc<AtomicUsize>,
    data: &'a mut T,
    idx: usize,
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

fn atomic_lock(lock: &Arc<AtomicUsize>, idx: usize) -> usize {
    lock.fetch_or(bitmask_lock(idx) | bitmask_wait(idx), Ordering::SeqCst)
}

fn atomic_unlock(lock: &Arc<AtomicUsize>, idx: usize) -> usize {
    lock.fetch_xor(bitmask_both(idx), Ordering::SeqCst)
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
    }
    
    fn set_lock(&self, idx: usize) -> usize {
        atomic_lock(&self.lock, idx)
    }
    
    fn get_reader_id(&self) -> usize {
        let r: usize = random();
        return r % READERS_MAX
    }
    
    fn writer_mask(&self) -> usize {
        return bitmask_both(WRITER_IDX);
    }
    
    pub fn obtain_reader_lock(&self) -> usize {
        let mut idx = self.get_reader_id();
        
        let mut prev_state = self.set_wait(idx);
        
        loop {
            if prev_state & bitmask_wait(idx) == 0 {
                break
            } else {
                idx = self.get_reader_id();
                cpu_relax();
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
    
    pub fn lock_reader(&self) -> ReadMutexGuard<T>
    {
        ReadMutexGuard {
            idx: self.obtain_reader_lock(),
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
    
    pub fn lock_writer(&self) -> WriteMutexGuard<T>
    {
        WriteMutexGuard {
            idx: self.obtain_writer_lock(),
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

macro_rules! define_deref_for {
    ($cls:path) => (
        impl<'a, T: ? Sized> Deref for ($cls)
        {
            type Target = T;
            fn deref<'b>(&'b self) -> &'b T { &*self.data }
        }
    )
}

macro_rules! define_deref_mut_for {
    ($cls:path) => (
        impl<'a, T: ? Sized> DerefMut for ($cls)
        {
            fn deref_mut<'b>(&'b mut self) -> &'b mut T {
                &mut *self.data
            }
        }
    )
}

macro_rules! define_drop_for {
    ($cls:path) => (
        impl<'a, T: ? Sized> Drop for ($cls)
        {
            fn drop(&mut self)
            {
                atomic_unlock(self.lock, self.idx);
            }
        }
    )
}

define_deref_for!(ReadMutexGuard<'a, T>);
define_deref_for!(WriteMutexGuard<'a, T>);
define_deref_mut_for!(WriteMutexGuard<'a, T>);
define_drop_for!(ReadMutexGuard<'a, T>);
define_drop_for!(WriteMutexGuard<'a, T>);