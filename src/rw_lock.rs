use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};

use std::sync::atomic::AtomicUsize;

use util::cpu_relax;
use helpers::*;
use arch::ARCH;

/// Provides single-writer multiple-reader lock based on a single atomic primitive
///
/// # Description
///
pub struct RwLock<T: ? Sized>
{
    lock: AtomicUsize,
    data: UnsafeCell<T>,
}

pub struct ReadLockGuard<'a, T: ? Sized + 'a>
{
    lock: &'a AtomicUsize,
    data: &'a T,
//        data: &'a UnsafeCell<T>,
    pub idx: usize,
}

pub struct WriteLockGuard<'a, T: ? Sized + 'a>
{
    lock: &'a AtomicUsize,
    data: &'a mut T,
//        data: &'a UnsafeCell<T>,
    idx: usize,
}

unsafe impl<T: ? Sized + Send> Sync for RwLock<T> {}

unsafe impl<T: ? Sized + Send> Send for RwLock<T> {}

pub struct LockMany<'a, T: ? Sized + 'a> {
    pub read: Vec<ReadLockGuard<'a, T>>,
    pub write: Vec<WriteLockGuard<'a, T>>,
}

impl<T> RwLock<T>
{
    //    fn state(&self) -> usize {
    //        atomic_load(&self.lock)
    //    }
    
    //    #[cfg(feature = "const_fn")]
    pub fn new(user_data: T) -> RwLock<T>
    {
        RwLock {
            lock: AtomicUsize::new(0),
            data: UnsafeCell::new(user_data),
        }
    }
    
    /// Locks all readers and writers at once. It's your responsibility that readers and writers
    /// do not overlap.
    ///
    /// # Arguments
    ///
    /// * `reader_idx` - an id for the readers (see examples)
    /// * `read` - a set of locks to be locked in reading mode
    /// * `write` - a set of locks to be lock in writing mode
    ///
    ///
    pub fn lock_many<'a>(reader_idx: usize, read: &Vec<&'a Self>, write: &Vec<&'a Self>) -> LockMany<'a, T> {
        // TODO: check if idx is < ARCH.reader_cnt
        
        let reader_idx = reader_idx % ARCH.reader_cnt;
        
        let mut read_locks = Vec::<&'a Self>::with_capacity(read.len());
        let mut write_locks = Vec::<&'a Self>::with_capacity(write.len());
        
        'root: for i in 0.. {
            if i > 0 {
                {
                    let read_locks = &mut read_locks;
                    let write_locks = &mut write_locks;
                    
                    for &mut r in read_locks {
                        atomic_reader_unlock(&r.lock, reader_idx);
                    }
                    
                    for &mut w in write_locks {
                        atomic_writer_unlock(&w.lock);
                    }
                }
                {
                    let read_locks = &mut read_locks;
                    let write_locks = &mut write_locks;
                    (*read_locks).clear();
                    (*write_locks).clear();
                }
                // TODO: We may want to increase the wait time here depending on the location.
                
                // TODO: this is a very picky one. 2-thread programs may sync quite well
                // TODO: and reduce the performance by 10-x.
                for y in 0..((read.len() + write.len()) * (reader_idx) * 10) {
                    cpu_relax();
                }
            }
            {
                let read_locks = &mut read_locks;
                let write_locks = &mut write_locks;
                
                for r in read {
                    let (_, owned, block) = atomic_reader_lock(&r.lock, reader_idx);
                    
                    if owned && block {
                        atomic_reader_unlock(&r.lock, reader_idx);
                        continue 'root
                    } else if !owned {
                        continue 'root
                    } else {
                        (*read_locks).push(r);
                    }
                }
                
                for w in write {
                    let (_, owned, block) = atomic_writer_lock(&w.lock);
                    if block {
                        atomic_writer_unlock(&w.lock);
                        continue 'root;
                    } else if !owned {
                        continue 'root
                    } else {
                        (*write_locks).push(w);
                    }
                }
            }
            
            break;
        }
        
        LockMany::<'a, T> {
            read: read_locks.iter().map(
                |args| {
                    let x = *args;
                    x.obtained_read(reader_idx)
                }).collect(),
            write: write_locks.iter().map(
                |args| {
                    let x = *args;
                    x.obtained_write(ARCH.reader_cnt)
                }).collect()
        }
    }
}

impl<T: ? Sized> RwLock<T>
{
    #[inline(always)]
    fn obtain_reader_lock(&self, idx: usize) -> usize {
        // TODO: check if idx is < ARCH.reader_cnt
        
        'root: loop {
            let (_, owned, block) = atomic_reader_lock(&self.lock, idx);
            if owned && !block {
                break
            } else if owned {
                loop {
                    while !atomic_writer_load(&self.lock) {
                        cpu_relax()
                    }
        
                    let (_, _, block) = atomic_reader_lock(&self.lock, idx);
        
                    if !block {
                        break 'root;
                    }
                }
            } else {
                while !atomic_reader_load(&self.lock, idx) {
                    cpu_relax()
                }
            }
        }
        
        return idx;
    }
    
    #[inline(always)]
    fn obtain_writer_lock(&self) -> usize {
        let idx = ARCH.reader_cnt;
        
        'root: loop {
            let (_, owned, block) = atomic_writer_lock(&self.lock);
            
            if owned && !block {
                break
            } else if owned {
                atomic_writer_unlock(&self.lock);
    
                while !atomic_readers_load(&self.lock) {
                    cpu_relax()
                }
            } else {
                while !atomic_writer_load(&self.lock) {
                    cpu_relax()
                }
            }
        }
        
        idx
    }
    
    fn obtained_read(&self, idx: usize) -> ReadLockGuard<T> {
        ReadLockGuard {
            idx: idx,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
//                        data: &self.data,
        }
    }
    
    fn obtained_write(&self, idx: usize) -> WriteLockGuard<T> {
        WriteLockGuard {
            idx: idx,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
//                        data: &self.data,
        }
    }
    
    /// Obtain the lock in read mode
    ///
    /// # Arguments
    /// * `idx` - reader index
    ///
    pub fn read(&self, idx: usize) -> ReadLockGuard<T>
    {
        self.obtained_read(self.obtain_reader_lock(idx))
    }
    
    /// Obtain the lock in write mode
    pub fn write(&self) -> WriteLockGuard<T>
    {
        self.obtained_write(self.obtain_writer_lock())
    }
}

macro_rules! define_deref_for {
    ($cls:path) => (
        impl<'a, T: ? Sized> Deref for ($cls)
        {
            type Target = T;
            fn deref<'b>(&'b self) -> &'b T { &*self.data }
//            fn deref<'b>(&'b self) -> &'b T { &*unsafe { &mut *(*self.data).get() } }
        }
    )
}

macro_rules! define_deref_mut_for {
    ($cls:path) => (
        impl<'a, T: ? Sized> DerefMut for ($cls)
        {
            fn deref_mut<'b>(&'b mut self) -> &'b mut T {
                &mut *self.data
//                 &mut *unsafe { &mut *(*self.data).get() }
            }
        }
    )
}

macro_rules! define_drop_for {
    ($cls:path) => (
        impl<'a, T: ? Sized> Drop for ($cls)
        {
            /// Can we, when the initialisation is being done
            fn drop(&mut self)
            {
//                println!("UNLOCK {}", self.idx);
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

//impl<'a, T> Clone for ReadLockGuard<'a, T> {
//    fn clone(&self) -> Self {
//        ReadLockGuard {
//            lock: self.lock,
//            data: self.data,
//            idx: self.idx
//        }
//    }
//
//    fn clone_from(&mut self, source: &Self) {
//        *self = source.clone()
//    }
//}
//
//impl<'a, T> Clone for WriteLockGuard<'a, T> {
//    fn clone(&self) -> Self {
//        WriteLockGuard {
//            lock: self.lock,
//            data: self.data,
//            idx: self.idx
//        }
//    }
//
//    fn clone_from(&mut self, source: &Self) {
//        *self = source.clone()
//    }
//}