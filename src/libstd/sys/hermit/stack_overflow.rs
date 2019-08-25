pub struct Handler;

impl Handler {
    pub unsafe fn new() -> Handler {
        Handler
    }
}

#[allow(dead_code)]
#[inline]
pub unsafe fn init() {
}

#[inline]
pub unsafe fn cleanup() {
}
