//! System bindings for HermitCore
//!
//! This module contains the facade (aka platform-specific) implementations of
//! OS level functionality for HermitCore.
//!
//! This is all super highly experimental and not actually intended for
//! wide/production use yet, it's still all in the experimental category. This
//! will likely change over time.
//!
//! Currently all functions here are basically stubs that immediately return
//! errors. The hope is that with a portability lint we can turn actually just
//! remove all this and just omit parts of the standard library if we're
//! compiling for wasm. That way it's a compile time error for something that's
//! guaranteed to be a runtime error!

use crate::os::raw::c_char;
use crate::intrinsics;

pub mod alloc;
pub mod args;
pub mod condvar;
pub mod stdio;
pub mod memchr;
pub mod io;
pub mod mutex;
pub mod rwlock;
pub mod os;
pub mod cmath;
pub mod thread;
pub mod env;
pub mod fs;
pub mod fd;
pub mod net;
pub mod path;
pub mod pipe;
pub mod process;
pub mod stack_overflow;
pub mod time;
pub mod thread_local;
pub mod fast_thread_local;

use core::{mem,slice};
pub use crate::sys_common::os_str_bytes as os_str;
use crate::io::ErrorKind;

pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::Error::new(crate::io::ErrorKind::Other,
           "operation not supported on HermitCore yet")
}

// This enum is used as the storage for a bunch of types which can't actually
// exist.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Void {}

pub unsafe fn strlen(mut s: *const c_char) -> usize {
    let mut n = 0;
    while *s != 0 {
        n += 1;
        s = s.offset(1);
    }
    return n
}

#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    unsafe {
        intrinsics::floorf64(x)
    }
}

pub unsafe fn abort_internal() -> ! {
    extern "C" {
        fn sys_abort() ->!;
    }

    sys_abort();
}

// TODO: just a workaround to test the system
pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub unsafe extern "C" fn __rust_abort() {
    abort_internal();
}

#[cfg(not(test))]
pub fn init() {
}

unsafe fn run_init_array(
    init_array_start: &extern "C" fn(),
    init_array_end: &extern "C" fn(),
) {
    let n = (init_array_end as *const _ as usize -
        init_array_start as *const _ as usize) /
        mem::size_of::<extern "C" fn()>();

    for f in slice::from_raw_parts(init_array_start, n) {
        f();
    }
}

#[cfg(not(test))]
#[no_mangle]
pub fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> ! {
    extern "C" {
        fn main();
        fn sys_exit(arg: i32) ->!;

        #[linkage = "extern_weak"]
        static __preinit_array_start: *const u8;
        #[linkage = "extern_weak"]
        static __preinit_array_end: *const u8;
        #[linkage = "extern_weak"]
        static __init_array_start: *const u8;
        #[linkage = "extern_weak"]
        static __init_array_end: *const u8;
    }

    unsafe {
        // run preinit array
        if __preinit_array_end as usize - __preinit_array_start as usize > 0 {
            run_init_array(mem::transmute::<&*const u8, &extern "C" fn()>(&__preinit_array_start), mem::transmute::<&*const u8, &extern "C" fn()>(&__preinit_array_end));
        }

        // run init array
        if __init_array_end as usize - __init_array_start as usize > 0 {
            run_init_array(mem::transmute::<&*const u8, &extern "C" fn()>(&__init_array_start), mem::transmute::<&*const u8, &extern "C" fn()>(&__init_array_end));
        }

        // initialize environment
        args::init(argc as isize, argv);
        os::init_environment(env);

        main();

        sys_exit(0);
    }
}

pub fn decode_error_kind(errno: i32) -> ErrorKind {
    match errno {
        x if x == 13 as i32 => ErrorKind::PermissionDenied,
        x if x == 98 as i32 => ErrorKind::AddrInUse,
        x if x == 99 as i32 => ErrorKind::AddrNotAvailable,
        x if x == 11 as i32 => ErrorKind::WouldBlock,
        x if x == 103 as i32 => ErrorKind::ConnectionAborted,
        x if x == 111 as i32 => ErrorKind::ConnectionRefused,
        x if x == 104 as i32 => ErrorKind::ConnectionReset,
        x if x == 17 as i32 => ErrorKind::AlreadyExists,
        x if x == 4 as i32 => ErrorKind::Interrupted,
        x if x == 22 as i32 => ErrorKind::InvalidInput,
        x if x == 2 as i32 => ErrorKind::NotFound,
        x if x == 107 as i32 => ErrorKind::NotConnected,
        x if x == 1 as i32 => ErrorKind::PermissionDenied,
        x if x == 32 as i32 => ErrorKind::BrokenPipe,
        x if x == 110 as i32 => ErrorKind::TimedOut,
        _ => ErrorKind::Other,
    }
}

pub fn cvt(result: i32) -> crate::io::Result<usize> {
    if result < 0 {
        Err(crate::io::Error::from_raw_os_error(-result))
    } else {
        Ok(result as usize)
    }
}
