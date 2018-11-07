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

fn unit_result(b: bool) -> Result<(), ()> {
    if b {
        Ok(())
    } else {
        Err(())
    }
}

#[test]
fn test_unit_result() {
    set_panic(true);
    #[allow(unused)]
    { black_box(unit_result(true)); }
    black_box(unit_result(true)).unwrap();
    #[allow(unused)]
    { black_box(unit_result(false)); }
    black_box(unit_result(false)).unwrap_err();
    set_panic(false);
}

#[test]
#[should_panic]
fn test_vec_push() {
    let mut v = Vec::new();
    set_panic(true);
    v.push(0);
}

#[test]
fn test_vec_push_capacity() {
    let mut v = Vec::with_capacity(1);
    set_panic(true);
    v.push(0);
    v.pop();
    v.push(0);
    set_panic(false);
}

#[test]
fn test_vec_with_zero() {
    set_panic(true);
    let _v: Vec<u8> = black_box(Vec::with_capacity(0));
    set_panic(false);
}

#[test]
fn test_vec_new() {
    set_panic(true);
    let _v: Vec<u8> = black_box(Vec::new());
    set_panic(false);
}

#[test]
#[should_panic]
fn test_vec_with_one() {
    set_panic(true);
    let _v: Vec<u8> = Vec::with_capacity(1);
}