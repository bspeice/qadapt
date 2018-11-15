extern crate qadapt;

#[test]
#[should_panic]
fn panic_not_using_qadapt() {
    ::qadapt::enter_protected();
}