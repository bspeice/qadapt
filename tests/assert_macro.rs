use qadapt::assert_no_alloc;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[test]
fn math() {
    let x = assert_no_alloc!(2 + 2);
    assert_eq!(x, 4);
}

fn early_return() -> usize {
    assert_no_alloc!(return 8)
}

fn into_box() -> Box<usize> {
    Box::new(early_return())
}

#[test]
#[should_panic]
fn early_return_boxing() {
    if cfg!(debug_assertions) {
        // The release-mode compiler is able to optimize through the Box
        into_box();
    } else {
        panic!("Intentional")
    }
}

