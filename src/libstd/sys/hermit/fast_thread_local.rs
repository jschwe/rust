/*#![cfg(target_thread_local)]
#![unstable(feature = "thread_local_internals", issue = "0")]
#![allow(dead_code)]

pub unsafe fn register_dtor(t: *mut u8, dtor: unsafe extern fn(*mut u8)) {
    extern {
        fn __thread_atexit(dtor: unsafe extern fn(*mut u8), arg: *mut u8) -> i32;
    }

    let _ = __thread_atexit(dtor, t);
}

pub fn requires_move_before_drop() -> bool {
    false
}*/

#![cfg(target_thread_local)]
#![unstable(feature = "thread_local_internals", issue = "0")]

pub use crate::sys_common::thread_local::register_dtor_fallback as register_dtor;
