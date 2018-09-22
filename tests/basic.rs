extern crate qadapt;

use qadapt::QADAPT;
use std::alloc::alloc;
use std::alloc::Layout;
use std::sync::atomic::Ordering;

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
    A.clear_allocations();
    assert!(!A.has_allocated());

    let _x = 24;
    assert!(!A.has_allocated());

    let _x = Empty {};
    assert!(!A.has_allocated());

    let _x = NonEmpty {
        _x: 42,
        _y: 84
    };
    assert!(!A.has_allocated());

    let _x = Box::new(42);
    assert!(A.has_allocated());
}