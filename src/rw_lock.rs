use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};

use std::sync::atomic::AtomicUsize;

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
//    data: &'a UnsafeCell<T>,
    idx: usize,
}

pub struct WriteLockGuard<'a, T: ? Sized + 'a>
{
    lock: &'a AtomicUsize,
    data: &'a mut T,
//    data: &'a UnsafeCell<T>,
    idx: usize,
}

unsafe impl<T: ? Sized + Send> Sync for RwLock<T> {}

unsafe impl<T: ? Sized + Send> Send for RwLock<T> {}

pub struct LockMany<'a, T: ? Sized + 'a> {
    read: Vec<ReadLockGuard<'a, T>>,
    write: Vec<WriteLockGuard<'a, T>>,
}

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
    
//    fn readers_lease<'a>(locks: &[&Self]) -> (&'a [(&'a Self, usize)], bool) {
//        let mut leases = Vec::<(&'a Self, usize, usize)>::with_capacity(locks.len());
//
//
//    }
    
//    pub fn read_lease<'a>(read: Vec<&'a Self>) -> (Vec<&'a Self, usize, usize>, bool) {
//
//    }
    
    /// Makes sure we obtain all of the reader and writer locks at once.
    /// Spinlocks conflicting elements.
    /// It's your responsibility to ensure that read and write do not intersect
    pub fn lock_many<'a>(read: Vec<&'a Self>, write: Vec<&'a Self>) -> LockMany<'a, T> {
        // We need to obtain the leases for writers and readers
        // We need to obtain the locks for writers and readers
        
        // Can we obtain the reader leases first ?
        
        
        // We first obtain all reader leases (means we've got enough capacity for reading).
        
        let mut read_leases = Vec::<(&'a Self, usize, usize)>::with_capacity(read.len());
        let mut write_leases = Vec::<(&'a Self, usize)>::with_capacity(write.len());
        
        for i in 0.. {
            if i > 0 {
                {
                    let read_leases = &mut read_leases;
                    let write_leases = &mut write_leases;
                    
                    for &mut (r, idx, _) in read_leases {
                        atomic_reader_unlease(&r.lock, idx);
                    }
        
                    for &mut (w, _) in write_leases {
                        atomic_writer_unlease(&w.lock);
                    }
                }
                {
                    let read_leases = &mut read_leases;
                    let write_leases = &mut write_leases;
                    (*read_leases).clear();
                    (*write_leases).clear();
                }
                
                cpu_relax();
            }
    
            {
                let read_leases = &mut read_leases;
                let write_leases = &mut write_leases;
                
                for r in read {
                    let idx = random_reader_idx();
                    let (prev_state, block) = atomic_reader_lease(&r.lock, idx);
            
                    if block {
                        continue
                    } else {
                        (*read_leases).push((r, idx, prev_state));
                    }
                }
        
                for w in write {
                    let (prev_state, block) = atomic_writer_lease(&w.lock);
                    if block {
                        continue
                    } else {
                        (*write_leases).push((w, prev_state));
                    }
                }
            }
            
            break;
        }
        
        let mut read_locks = Vec::<(&'a Self, usize, usize)>::with_capacity(read_leases.len());
        let mut write_locks = Vec::<(&'a Self, usize)>::with_capacity(write_leases.len());

        for i in 0.. {
            if i > 0 {
                {
                    let read_leases = &mut read_leases;
                    let write_leases = &mut write_leases;
        
                    for &mut (r, idx, _) in read_leases {
                        atomic_reader_unlock(&r.lock, idx);
                    }
        
                    for &mut (w, _) in write_leases {
                        atomic_writer_unlock(&w.lock);
                    }
                }
                {
                    let read_locks = &mut read_locks;
                    let write_locks = &mut write_locks;
                    (*read_locks).clear();
                    (*write_locks).clear();
                }
                cpu_relax();
            }
            {
                let read_locks = &mut read_locks;
                let write_locks = &mut write_locks;

                for (r, idx, _) in read_leases {
                    let (prev_state, block) = atomic_reader_lock(&r.lock, idx);

                    if block {
                        continue
                    } else {
                        (*read_locks).push((r, idx, prev_state));
                    }
                }

                for (w, _) in write_leases {
                    let (prev_state, block) = atomic_writer_lock(&w.lock);
                    if block {
                        continue
                    } else {
                        (*write_locks).push((w, prev_state))
                    }
                }
            }

            break;
        }
        
        LockMany::<'a, T> {
//            read: Vec::new()
//            ,
            read: read_locks.iter().map(
                |args| {
                    let (x, y, _) = *args;
                    x.obtained_read(y)
                }).collect(),
//            write: Vec::new()
            write: write_locks.iter().map(
                |args| {
                    let (x, y) = *args;
                    x.obtained_write(ARCH.reader_cnt)
                }).collect()
        }
    }
}

impl<T: ? Sized> RwLock<T>
{
    fn obtain_reader_lock(&self) -> usize {
        let mut idx = random_reader_idx();
        
        loop {
            let (mut prev_state, block) = atomic_reader_lease(&self.lock, idx);
            if block {
                idx = random_reader_idx();
                cpu_relax();
            } else {
                break
            }
        }
        
        loop {
            let (prev_state, block) = atomic_reader_lock(&self.lock, idx);
            if block {
                atomic_reader_unlock(&self.lock, idx);
                cpu_relax()
            } else {
                break
            }
        }
        
        return idx;
    }
    
    fn obtain_writer_lock(&self) -> usize {
        let idx = ARCH.reader_cnt;
        
        loop {
            let (prev_state, block) = atomic_writer_lease(&self.lock);
            if block {
                cpu_relax();
            } else {
                break
            }
        }
        
        loop {
            let (prev_state, block) = atomic_writer_lock(&self.lock);
            
            if block {
                atomic_writer_unlock(&self.lock);
                cpu_relax()
            } else {
                break
            }
        }
        
        idx
    }
    
    fn obtained_read(&self, idx: usize) -> ReadLockGuard<T> {
        ReadLockGuard {
            idx: idx,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
//            data: &self.data,

        }
    }
    
    fn obtained_write(&self, idx: usize) -> WriteLockGuard<T> {
        WriteLockGuard {
            idx: idx,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
//            data: &self.data,
        }
    }
    
    pub fn read(&self) -> ReadLockGuard<T>
    {
        self.obtained_read(self.obtain_reader_lock())
    }
    
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