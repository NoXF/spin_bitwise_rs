use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};

use std::sync::atomic::{AtomicUsize, Ordering};

use util::cpu_relax;
use helpers::*;
use arch::ARCH;


pub struct RwLock<T: ? Sized>
{
    lock: AtomicUsize,
    data: UnsafeCell<T>,
}

pub struct ReadLockGuard<'a, T: ? Sized + 'a>
{
    lock: &'a AtomicUsize,
    data: &'a T,
    idx: usize,
}

pub struct WriteLockGuard<'a, T: ? Sized + 'a>
{
    lock: &'a AtomicUsize,
    data: &'a mut T,
    idx: usize,
}

unsafe impl<T: ? Sized + Send> Sync for RwLock<T> {}

unsafe impl<T: ? Sized + Send> Send for RwLock<T> {}

impl<T> RwLock<T>
{
    //    #[cfg(feature = "const_fn")]
    pub fn new(user_data: T) -> RwLock<T>
    {
        RwLock {
            lock: AtomicUsize::new(0),
            data: UnsafeCell::new(user_data),
        }
    }
}

fn atomic_load(lock: &AtomicUsize) -> usize {
    lock.load(Ordering::SeqCst)
}

impl<T: ? Sized> RwLock<T>
{
    fn obtain_reader_lock(&self) -> usize {
        let mut idx = random_reader_idx();
        
        let mut prev_state = atomic_wait(&self.lock, idx);
        
        loop {
            if prev_state & bitmask_wait(idx) == 0 {
                break
            } else {
                // TODO: We might check the ARCH.reader_wait_mask in order to tell which slots are
                // TODO: available.
                idx = random_reader_idx();
                cpu_relax();
            }
            
            prev_state = atomic_wait(&self.lock, idx);
        }
        
        loop {
            if prev_state & bitmask_writer() == 0 {
                break
            } else {
                prev_state = atomic_load(&self.lock)
            }
            
            cpu_relax();
        }
        
        atomic_lock(&self.lock, idx);
        
        return idx;
    }
    
    fn obtain_writer_lock(&self) -> usize {
        let idx = ARCH.reader_cnt;
        
        loop {
            let state = atomic_wait(&self.lock, idx);
            
            if state & bitmask_readers_lock() == 0 {
                break;
            }
            
            cpu_relax();
        }
        loop {
            if atomic_lock(&self.lock, idx) & bitmask_lock(idx) == 0 {
                break;
            }
            
            cpu_relax();
        }
        
        idx
    }
    
    pub fn read(&self) -> ReadLockGuard<T>
    {
        ReadLockGuard {
            idx: self.obtain_reader_lock(),
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
    
    pub fn write(&self) -> WriteLockGuard<T>
    {
        WriteLockGuard {
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

define_deref_for!(ReadLockGuard<'a, T>);
define_deref_for!(WriteLockGuard<'a, T>);
define_deref_mut_for!(WriteLockGuard<'a, T>);
define_drop_for!(ReadLockGuard<'a, T>);
define_drop_for!(WriteLockGuard<'a, T>);