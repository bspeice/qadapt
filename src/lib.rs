#![no_std]

extern crate libc;
extern crate spin;

use libc::c_void;
use libc::free;
use libc::malloc;
use core::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

mod const_init;
use const_init::ConstInit;

static INTERNAL_ALLOCATION: AtomicBool = AtomicBool::new(false);

pub struct QADAPTInternal {
    pub has_allocated: AtomicBool
}

pub struct QADAPT {
    internal: spin::Once<QADAPTInternal>
}

impl ConstInit for QADAPT {
    const INIT: QADAPT = QADAPT {
        internal: spin::Once::new()
    };
}

unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !INTERNAL_ALLOCATION.load(Ordering::SeqCst) {
            self.internal().has_allocated.store(true, Ordering::SeqCst);
        }

        malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void)
    }
}

impl QADAPT {
    pub const INIT: Self = <Self as ConstInit>::INIT;

    fn internal(&self) -> &QADAPTInternal {

        self.internal.call_once(|| {
            INTERNAL_ALLOCATION.store(true, Ordering::SeqCst);
            let q = QADAPTInternal {
                has_allocated: AtomicBool::new(false)
            };
            INTERNAL_ALLOCATION.store(false, Ordering::SeqCst);

            q
        })
    }

    pub fn clear_allocations(&self) {
        self.internal().has_allocated.store(false, Ordering::SeqCst);
    }

    pub fn has_allocated(&self) -> bool {
        self.internal().has_allocated.load(Ordering::SeqCst)
    }
}
