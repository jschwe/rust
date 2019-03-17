use crate::ffi::OsString;
//use crate::sys_common::FromInner;
use crate::slice;

#[allow(dead_code)]
pub fn init(_: isize, _: *const *const u8) {}

#[allow(dead_code)]
pub fn cleanup() {}

/// Returns the command line arguments
pub fn args() -> Args {
    Args([].iter())
}

pub struct Args(slice::Iter<'static, OsString>);

impl Args {
    pub fn inner_debug(&self) -> &[OsString] {
        self.0.as_slice()
    }
}

impl Iterator for Args {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        self.0.next().cloned()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        self.0.next_back().cloned()
    }
}
