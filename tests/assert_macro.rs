use qadapt::assert_no_alloc;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[test]
fn math() {
    let x = assert_no_alloc!(2 + 2);
    assert_eq!(x, 4);
}

// Because the `exit_protected` guard is never run, the compiler
// warns us of unreachable code
#[allow(unreachable_code)]
fn early_return() -> usize {
    assert_no_alloc!(return 8)
}

#[test]
#[should_panic]
fn early_return_boxing() {
    // The release-mode compiler is able to optimize through the Box
    if cfg!(debug_assertions) {
        Box::new(early_return());
    } else {
        panic!("Intentional")
    }
}

#[test]
#[should_panic]
fn list_push() {
    let mut x = Vec::with_capacity(1);
    x.push(1);

    if cfg!(debug_assertions) {
        assert_no_alloc!(x.push(12))
    } else {
        panic!("Intentional")
    }
}