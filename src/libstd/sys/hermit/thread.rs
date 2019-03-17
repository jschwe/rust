use crate::boxed::FnBox;
use crate::ffi::CStr;
use crate::io;
use crate::sys::{unsupported, Void};
use crate::time::Duration;

pub struct Thread(Void);

pub const DEFAULT_MIN_STACK_SIZE: usize = 4096;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(_stack: usize, _p: Box<dyn FnBox()>)
        -> io::Result<Thread>
    {
        unsupported()
    }

    pub fn yield_now() {
        // do nothing
    }

    pub fn set_name(_name: &CStr) {
        // nope
    }

    #[cfg(not(target_feature = "atomics"))]
    pub fn sleep(_dur: Duration) {
        panic!("can't sleep");
    }

    #[cfg(target_feature = "atomics")]
    pub fn sleep(_dur: Duration) {
        //use arch::wasm32;
        //use cmp;

        // Use an atomic wait to block the current thread artificially with a
        // timeout listed. Note that we should never be notified (return value
        // of 0) or our comparison should never fail (return value of 1) so we
        // should always only resume execution through a timeout (return value
        // 2).
        /*let mut nanos = dur.as_nanos();
        while nanos > 0 {
            let amt = cmp::min(i64::max_value() as u128, nanos);
            let mut x = 0;
            let val = unsafe { wasm32::i32_atomic_wait(&mut x, 0, amt as i64) };
            debug_assert_eq!(val, 2);
            nanos -= amt;
        }*/
    }

    pub fn join(self) {
        match self.0 {}
    }
}

pub mod guard {
    pub type Guard = !;
    pub unsafe fn current() -> Option<Guard> { None }
    pub unsafe fn init() -> Option<Guard> { None }
}

/*pub fn my_id() -> u32 {
    panic!("thread ids not implemented on wasm with atomics yet")
}

pub fn tcb_get() -> *mut u8 {
    panic!("thread local data not implemented on wasm with atomics yet")
}

pub fn tcb_set(_ptr: *mut u8) {
    panic!("thread local data not implemented on wasm with atomics yet")
}*/
