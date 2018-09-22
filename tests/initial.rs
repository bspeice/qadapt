extern crate qadapt;

use qadapt::QADAPT;
use std::sync::atomic::Ordering;

#[global_allocator]
static A: QADAPT = QADAPT::INIT;

#[test]
fn init() {
    // Because the Allocator and its internals isn't the only "pre-main" allocation
    // that happens, when starting up we expect to see that A has in fact allocated
    assert!(A.has_allocated());

    A.clear_allocations();
    assert!(!A.has_allocated());

    let _x = Box::new(42);
    assert!(A.has_allocated());
}