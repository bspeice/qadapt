extern crate qadapt;
use std::io;

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

#[allocate_panic]
fn no_allocate_ret() -> bool {
    return true;
}

#[allocate_panic]
fn no_allocate_implicit_ret() -> bool {
    true
}

#[allocate_panic]
fn no_allocate_arg(b: bool) -> bool {
    b
}

#[allocate_panic]
fn no_allocate_args(_b: bool, _u: usize, i: i64) -> i64 {
    i
}

#[allocate_panic]
fn return_result(r: Result<usize, io::Error>) -> Result<Result<usize, io::Error>, ()> {
    Ok(r)
}

#[test]
fn macro_no_allocate() {
    no_allocate();
}

#[test]
#[should_panic]
fn macro_allocates() {
    allocates();
}

#[test]
fn macro_return() {
    assert!(no_allocate_ret());
}

#[test]
fn macro_implicit_return() {
    assert!(no_allocate_ret());
}

#[test]
fn macro_allocate_arg() {
    no_allocate_arg(true);
    no_allocate_arg(false);
}

#[test]
fn macro_allocate_args() {
    no_allocate_args(true, 0, -1);
    no_allocate_args(false, 4, -90);
}

#[test]
fn macro_return_result() {
    return_result(Ok(16)).unwrap().unwrap();
}