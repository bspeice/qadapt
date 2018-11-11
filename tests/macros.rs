extern crate qadapt;

use qadapt::allocate_panic;

#[allocate_panic]
fn allocates() {
    let _v: Vec<()> = Vec::with_capacity(1);
}

#[allocate_panic]
fn no_allocate() {
    let _v: Vec<()> = Vec::with_capacity(0);
}

#[test]
fn test_no_allocate() {
    no_allocate();
}

/*
#[test]
#[should_panic]
fn test_allocates() {
    allocates();
}
*/
