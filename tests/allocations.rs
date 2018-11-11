#![feature(asm)]
extern crate qadapt;

use qadapt::enter_protected;
use qadapt::exit_protected;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

pub fn black_box<T>(dummy: T) -> T {
    // Taken from test lib, need to mark the arg as non-introspectable
    unsafe { asm!("" : : "r"(&dummy)) }
    dummy
}

#[test]
fn test_copy() {
    enter_protected();
    black_box(0u8);
    exit_protected();
}

#[test]
#[should_panic]
fn test_allocate() {
    enter_protected();
    let _x = Box::new(12);
    exit_protected();
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
    enter_protected();
    #[allow(unused)]
    {
        black_box(unit_result(true));
    }
    black_box(unit_result(true)).unwrap();
    #[allow(unused)]
    {
        black_box(unit_result(false));
    }
    black_box(unit_result(false)).unwrap_err();
    exit_protected();
}

#[test]
#[should_panic]
fn test_vec_push() {
    let mut v = Vec::new();
    enter_protected();
    v.push(0);
}

#[test]
fn test_vec_push_capacity() {
    let mut v = Vec::with_capacity(1);
    enter_protected();
    v.push(0);
    v.pop();
    v.push(0);
    exit_protected();
}

#[test]
fn test_vec_with_zero() {
    enter_protected();
    let _v: Vec<u8> = black_box(Vec::with_capacity(0));
    exit_protected();
}

#[test]
fn test_vec_new() {
    enter_protected();
    let _v: Vec<u8> = black_box(Vec::new());
    exit_protected();
}

#[test]
#[should_panic]
fn test_vec_with_one() {
    enter_protected();
    let _v: Vec<u8> = Vec::with_capacity(1);
}

#[test]
#[should_panic]
fn exit_too_often() {
    enter_protected();
    exit_protected();
    exit_protected();
}
