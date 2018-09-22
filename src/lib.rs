extern crate libc;

use libc::c_void;
use libc::free;
use libc::malloc;
use std::alloc::Layout;
use std::alloc::GlobalAlloc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

mod const_init;
use const_init::ConstInit;

pub struct QADAPT {
    pub has_allocated: AtomicBool
}

impl ConstInit for QADAPT {

    const INIT: QADAPT = QADAPT {
        has_allocated: AtomicBool::new(false)
    };
}

unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let block = malloc(layout.size()) as *mut u8;
        self.has_allocated.store(true, Ordering::SeqCst);

        block
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void)
    }
}

impl QADAPT {
    pub const INIT: Self = <Self as ConstInit>::INIT;

    pub fn clear_allocations(&self) {
        self.has_allocated.store(false, Ordering::Release)
    }
}
