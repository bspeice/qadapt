#![no_std]
#![feature(alloc)]

extern crate alloc;
extern crate libc;
extern crate spin;

use alloc::collections::btree_map::BTreeMap;
use libc::c_void;
use libc::free;
use libc::malloc;
use core::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use spin::RwLock;

mod const_init;
use const_init::ConstInit;

mod thread_id;

// TODO: Doesn't check for race conditions
static INTERNAL_ALLOCATION: AtomicBool = AtomicBool::new(false);

pub struct QADAPTInternal {
    pub thread_has_allocated: BTreeMap<usize, AtomicBool>,
    pub recording_enabled: BTreeMap<usize, AtomicBool>
}

pub struct QADAPT {
    internal: spin::Once<RwLock<QADAPTInternal>>
}

impl ConstInit for QADAPT {
    const INIT: QADAPT = QADAPT {
        internal: spin::Once::new()
    };
}



unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !INTERNAL_ALLOCATION.load(Ordering::SeqCst) {
            let tid = thread_id::get();

            // Need to use RAII guard because record_allocation() needs write access
            let should_record = {
                let internal = self.internal().read();
                internal.recording_enabled.contains_key(&tid)
                    && internal.recording_enabled.get(&tid).unwrap().load(Ordering::SeqCst)
            };

            if should_record {
                self.record_allocation(thread_id::get())
            }
        }

        malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void)
    }
}

impl QADAPT {
    pub const INIT: Self = <Self as ConstInit>::INIT;

    fn internal(&self) -> &RwLock<QADAPTInternal> {

        self.internal.call_once(|| {
            INTERNAL_ALLOCATION.store(true, Ordering::SeqCst);
            let q = QADAPTInternal {
                thread_has_allocated: BTreeMap::new(),
                recording_enabled: BTreeMap::new()
            };
            INTERNAL_ALLOCATION.store(false, Ordering::SeqCst);

            RwLock::new(q)
        })
    }

    pub fn reset_allocation_state(&self) {
        let internal = self.internal().write();
        for (_tid, has_allocated) in &internal.thread_has_allocated {

            has_allocated.store(false, Ordering::SeqCst);
        }
        for (_tid, recording_enabled) in &internal.recording_enabled {

            recording_enabled.store(false, Ordering::SeqCst);
        }
    }

    pub fn has_allocated_current(&self) -> bool {
        let tid = thread_id::get();
        let internal = self.internal().read();

        // UNWRAP: Already checked for existence
        internal.thread_has_allocated.contains_key(&tid)
            && internal.thread_has_allocated.get(&tid).unwrap().load(Ordering::SeqCst)
    }

    pub fn record_allocation(&self, thread_id: usize) {
        let mut internal = self.internal().write();
        if internal.thread_has_allocated.contains_key(&thread_id) {
            // UNWRAP: Already checked for existence
            internal.thread_has_allocated.get(&thread_id)
                .unwrap().store(true, Ordering::SeqCst)
        }
        else {
            INTERNAL_ALLOCATION.store(true, Ordering::SeqCst);
            internal.thread_has_allocated.insert(thread_id, AtomicBool::new(true));
            INTERNAL_ALLOCATION.store(false, Ordering::SeqCst);
        }
    }

    pub fn enable_recording_current(&self) {
        self.enable_recording(thread_id::get());
    }

    pub fn enable_recording(&self, tid: usize) {
        let mut internal = self.internal().write();

        if internal.recording_enabled.contains_key(&tid) {
            // UNWRAP: Already checked for existence
            internal.recording_enabled.get(&tid).unwrap().store(true, Ordering::SeqCst);
        }
        else {
            INTERNAL_ALLOCATION.store(true, Ordering::SeqCst);
            internal.recording_enabled.insert(tid, AtomicBool::new(true));
            INTERNAL_ALLOCATION.store(false, Ordering::SeqCst);
        }
    }
}
