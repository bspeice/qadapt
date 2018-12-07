use qadapt::enter_protected;

#[test]
#[should_panic]
fn guard_without_initialization() {
    enter_protected();
}