#![cfg(target_thread_local)]
#![unstable(feature = "thread_local_internals", issue = "0")]

pub unsafe fn register_dtor(_t: *mut u8, _dtor: unsafe extern fn(*mut u8)) {
}

pub fn requires_move_before_drop() -> bool {
    false
}
