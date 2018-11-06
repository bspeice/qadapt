#![feature(asm)]

extern crate qadapt;

use qadapt::QADAPT;
use qadapt::set_panic;

#[global_allocator]
static Q: QADAPT = QADAPT;

pub fn black_box<T>(dummy: T) -> T {
    // Taken from test lib, need to mark the arg as non-introspectable
    unsafe {asm!("" : : "r"(&dummy))}
    dummy
}

#[test]
fn test_copy() {
    set_panic(true);
    black_box(0u8);
    set_panic(false);
}

#[test]
#[should_panic]
fn test_allocate() {
    set_panic(true);
    let _x = Box::new(12);
    set_panic(false);
}