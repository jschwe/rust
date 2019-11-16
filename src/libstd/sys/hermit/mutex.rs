use crate::sys::hermit::abi;

pub struct Mutex {
    inner: abi::Semaphore
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex { inner: abi::Semaphore::new(1) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
    }

    #[inline]
    pub unsafe fn lock(&self) {
        let _ = self.inner.acquire(None);
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let _ = self.inner.release();
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
    inner: abi::RecursiveMutex
}

impl ReentrantMutex {
    pub unsafe fn uninitialized() -> ReentrantMutex {
        ReentrantMutex { inner: abi::RecursiveMutex::new() }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
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
        self.inner.release();
    }

    #[inline]
    pub unsafe fn destroy(&self) {
    }
}
