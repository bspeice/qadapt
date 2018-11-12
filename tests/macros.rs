extern crate qadapt;

use qadapt::allocate_panic;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[allocate_panic]
fn allocates() {
    assert_eq!(::qadapt::protection_level(), 1);
    // Without boxing, release profile can actually optimize out the allocation
    let mut v = Box::new(Vec::new());
    v.push(1);
}

#[allocate_panic]
fn no_allocate() {
    assert_eq!(::qadapt::protection_level(), 1);
    let _v: Vec<()> = Vec::with_capacity(0);
}

#[test]
fn test_no_allocate() {
    no_allocate();
}

#[test]
#[should_panic]
fn test_allocates() {
    allocates();
}
