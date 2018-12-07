use qadapt::enter_protected;

#[test]
#[should_panic]
fn guard_without_initialization() {
    if cfg!(debug_assertions) {
        enter_protected();
    } else {
        panic!("Intentional")
    }
}
