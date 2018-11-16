extern crate qadapt;

use qadapt::enter_protected;
use qadapt::exit_protected;
use qadapt::protection_level;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[test]
fn test_copy() {
    enter_protected();
    let v = 0u8;
    let _v2 = v;
    exit_protected();
}

#[test]
#[should_panic]
fn test_allocate() {
    enter_protected();
    let _x = Box::new(12);
    exit_protected();
}

fn return_unit_result(b: bool) -> Result<(), ()> {
    if b {
        Ok(())
    } else {
        Err(())
    }
}

#[test]
fn unit_result() {
    enter_protected();
    return_unit_result(true).unwrap();
    return_unit_result(false).unwrap_err();
    exit_protected();
}

#[test]
#[should_panic]
fn vec_push() {
    let mut v = Vec::new();
    enter_protected();
    v.push(0);
    // We don't make it here in debug mode, but in release mode,
    // pushing one element doesn't trigger an allocation. Instead,
    // we use a box to force it onto the heap
    assert_eq!(protection_level(), 1);
    let _b = Box::new(v);
}

#[test]
fn vec_push_capacity() {
    let mut v = Vec::with_capacity(1);
    enter_protected();
    v.push(0);
    v.pop();
    v.push(0);
    exit_protected();
}

#[test]
fn vec_with_zero() {
    enter_protected();
    let _v: Vec<u8> = Vec::with_capacity(0);
    exit_protected();
}

#[test]
fn vec_new() {
    enter_protected();
    let _v: Vec<u8> = Vec::new();
    exit_protected();
}

#[test]
#[should_panic]
fn vec_with_one() {
    enter_protected();
    let v: Vec<u8> = Vec::with_capacity(1);
    // We don't make it here in debug mode, but in release mode,
    // pushing one element doesn't trigger an allocation. Instead,
    // we use a box to force it onto the heap
    assert_eq!(protection_level(), 1);
    let _b = Box::new(v);
}

#[test]
#[should_panic]
fn exit_too_often() {
    enter_protected();
    exit_protected();
    exit_protected();
}

#[test]
#[should_panic]
fn intentional_drop() {
    let v: Vec<()> = Vec::new();
    let v = Box::new(v);
    enter_protected();
    drop(v);
}
