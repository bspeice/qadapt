extern crate qadapt;

use qadapt::QADAPT;
use std::sync::atomic::Ordering;

#[global_allocator]
static A: QADAPT = QADAPT::INIT;

#[test]
fn init() {
    // Make sure that we don't have any allocations at the start
    // that pollute other tests
    assert!(!A.has_allocated.load(Ordering::SeqCst));
}