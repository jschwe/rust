#![allow(dead_code)]

use crate::ffi::CStr;
use crate::io;
use crate::time::Duration;
use crate::mem;
use core::u32;
use hermit::syscalls::{Tid,sys_usleep,sys_yield,sys_spawn};
use hermit::scheduler::task::{Priority,NORMAL_PRIO};

use crate::sys_common::thread::*;

pub struct Thread {
    tid: Tid
}

pub const DEFAULT_MIN_STACK_SIZE: usize = hermit::DEFAULT_STACK_SIZE;

impl Thread {
    pub unsafe fn new(_stack: usize, p: Box<dyn FnOnce()>)
        -> io::Result<Thread>
    {
        let mut tid: Tid = u32::MAX;
        let ret = sys_spawn(&mut tid as *mut Tid, thread_start, &*p as *const _ as *const u8 as usize,
                            Priority::into(NORMAL_PRIO), 0);

        return if ret == 0 {
            Ok(Thread { tid: tid })
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Unable to create thread!"))
        };

        extern fn thread_start(main: usize) {
            unsafe {
                start_thread(main as *mut u8);
            }
        }
    }

    #[inline]
    pub fn yield_now() {
        sys_yield();
    }

    #[inline]
    pub fn set_name(_name: &CStr) {
        // nope
    }

    #[inline]
    pub fn sleep(dur: Duration) {
        sys_usleep(dur.as_micros() as u64);
    }

    pub fn join(self) {
        //match self.0 {}
    }

    #[inline]
    pub fn id(&self) -> Tid { self.tid }

    #[inline]
    pub fn into_id(self) -> Tid {
        let id = self.tid;
        mem::forget(self);
        id
    }
}

pub mod guard {
    pub type Guard = !;
    pub unsafe fn current() -> Option<Guard> { None }
    pub unsafe fn init() -> Option<Guard> { None }
}
