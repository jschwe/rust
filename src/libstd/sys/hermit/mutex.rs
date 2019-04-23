use crate::mem;
use hermit::synch::recmutex::*;
use hermit::synch::semaphore::*;

pub struct Mutex {
    inner: Semaphore
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex { inner: Semaphore::new(1) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
    }

    #[inline]
    pub unsafe fn lock(&self) {
        self.inner.acquire(None);
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        self.inner.release();
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        self.inner.try_acquire()
    }

    #[inline]
    pub unsafe fn destroy(&self) {
    }
}

pub struct ReentrantMutex {
    inner: RecursiveMutex
}

impl ReentrantMutex {
    pub unsafe fn uninitialized() -> ReentrantMutex {
        ReentrantMutex { inner: mem::uninitialized() }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.inner = RecursiveMutex::new()
    }

    #[inline]
    pub unsafe fn lock(&self) {
        self.inner.acquire();
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        true
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        self.inner.release()
    }

    #[inline]
    pub unsafe fn destroy(&self) {}
}
