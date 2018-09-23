extern crate qadapt;

use qadapt::QADAPT;

#[global_allocator]
static A: QADAPT = QADAPT::INIT;

#[test]
fn init() {
    assert!(!A.has_allocated_current());
    A.reset_allocation_state();
    A.enable_recording_current();

    assert!(!A.has_allocated_current());

    let _x = Box::new(42);
    assert!(A.has_allocated_current());
}