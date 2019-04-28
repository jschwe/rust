use crate::mem;
use hermit::synch::recmutex::*;
use hermit::synch::semaphore::*;

pub struct Mutex {
    inner: Option<Semaphore>
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex { inner: None }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.inner = Some(Semaphore::new(1));
    }

    #[inline]
    pub unsafe fn lock(&self) {
        match &self.inner {
            Some(b) => { let _ = b.acquire(None); },
            None => panic!("Usage of an uninitialized mutex")
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        match &self.inner {
            Some(b) => b.release(),
            None => panic!("Usage of an uninitialized mutex")
        }
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        match &self.inner {
            Some(b) => b.try_acquire(),
            None => panic!("Usage of an uninitialized mutex")
        }
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
