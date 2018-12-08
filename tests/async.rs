use futures::future::ok;
use futures::prelude::*;
use qadapt::assert_no_alloc;
use qadapt::no_alloc;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[no_alloc]
fn async_box() -> impl Future<Item = Box<u8>, Error = ()> {
    ok(12).and_then(|e| Ok(Box::new(e)))
}

#[test]
fn raw_call() {
    async_box();
}

#[test]
fn guarded_call() {
    assert_no_alloc!(async_box());
}

#[test]
#[should_panic]
fn guarded_poll() {
    if cfg!(debug_assertions) {
        assert_no_alloc!(async_box().poll().unwrap());
    } else {
        panic!("Intentional")
    }
}
