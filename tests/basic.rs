extern crate qadapt;

use qadapt::QADAPT;
use std::alloc::alloc;
use std::alloc::Layout;

#[global_allocator]
static A: QADAPT = QADAPT::INIT;

#[test]
fn alloc_nonnull() {
    unsafe {
        assert!(!alloc(Layout::new::<u32>()).is_null())
    }
}

struct Empty;

struct NonEmpty {
    _x: i32,
    _y: i32
}

#[test]
fn allocation_flag() {
    A.reset_allocation_state();
    A.enable_recording_current();
    assert!(!A.has_allocated_current());

    let _x = 24;
    assert!(!A.has_allocated_current());

    let _x = Empty {};
    assert!(!A.has_allocated_current());

    let _x = NonEmpty {
        _x: 42,
        _y: 84
    };
    assert!(!A.has_allocated_current());

    let _x = Box::new(42);
    assert!(A.has_allocated_current());
}

#[inline(never)]
fn no_op() {}

#[test]
fn no_alloc_during_noop() {
    A.reset_allocation_state();
    A.enable_recording_current();
    assert!(!A.has_allocated_current());

    no_op();
    assert!(!A.has_allocated_current());
}

#[inline(never)]
fn allocates() {
    let _x = Box::new(42);
}

#[test]
fn alloc_during_func_call() {
    A.reset_allocation_state();
    A.enable_recording_current();
    assert!(!A.has_allocated_current());

    allocates();
    assert!(A.has_allocated_current());
}

#[test]
fn allocates_unrecorded() {
    A.reset_allocation_state();
    assert!(!A.has_allocated_current());

    allocates();
    assert!(!A.has_allocated_current());
}