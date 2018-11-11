//! The Quick And Dirty Allocation Profiling Tool
//! 
//! This allocator is a helper for writing high-performance code that is allocation/drop free;
//! for functions annotated with `#[allocate_panic]`, QADAPT will detect when allocations/drops
//! happen during their execution (and execution of any functions they call) and throw a
//! thread panic if this occurs.
//! 
//! Because QADAPT panics on allocation and is rather slow (for an allocator) it is **strongly**
//! recommended that QADAPT (the allocator) be used only in code tests. Functions annotated with
//! `#[allocate_panic]` will have no side effects if the QADAPT allocator is not being used,
//! so the attribute is safe to leave everywhere.
#![deny(missing_docs)]
extern crate libc;
extern crate qadapt_macro;
extern crate spin;

// Re-export the proc macros to use by other code
pub use qadapt_macro::*;

use libc::c_void;
use libc::free;
use libc::malloc;
use libc::pthread_self;
use spin::RwLock;
use std::alloc::Layout;
use std::alloc::GlobalAlloc;
use std::thread;

thread_local! {
    static PROTECTION_LEVEL: RwLock<u32> = RwLock::new(0);
}

/// The QADAPT allocator itself
pub struct QADAPT;

/// Let QADAPT know that we are now entering a protected region and that
/// panics should be triggered if allocations/drops happen while we are running.
pub fn enter_protected() {
    if thread::panicking() {
        return
    }

    PROTECTION_LEVEL.try_with(|v| {
        *v.write() += 1;
    }).unwrap_or_else(|_e| ());
}

/// Let QADAPT know that we are exiting a protected region. Will panic
/// if we attempt to [`exit_protected`] more times than we [`enter_protected`].
pub fn exit_protected() {
    if thread::panicking() {
        return
    }

    PROTECTION_LEVEL.try_with(|v| {
        let val = { *v.read() };
        match val {
            v if v == 0 => panic!("Attempt to exit protected too many times"),
            _ => {
                *v.write() -= 1;
            }
        }
    }).unwrap_or_else(|_e| ());
}

static INTERNAL_ALLOCATION: RwLock<u64> = RwLock::new(u64::max_value());

unsafe fn claim_internal_alloc() -> u64 {
    // std::thread::current() isn't safe because of thread-local allocations
    let tid = pthread_self();
    loop {
        match INTERNAL_ALLOCATION.write() {
            ref mut lock if **lock == u64::max_value() => {
                **lock = tid;
                break
            },
            _ => ()
        }
    }

    tid
}

unsafe fn release_internal_alloc() -> u64 {
    let tid = pthread_self();
    // TODO: Potential issues with releasing lock too early?
    match INTERNAL_ALLOCATION.write() {
        ref mut lock if **lock == tid => **lock = u64::max_value(),
        _ => panic!("Internal allocation tracking error")
    }

    tid
}

unsafe fn alloc_immediate() -> bool {
    thread::panicking() || *INTERNAL_ALLOCATION.read() == pthread_self()
}

unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // If we're attempting to allocate our PROTECTION_LEVEL thread local,
        // just allow it through
        if alloc_immediate() {
            return malloc(layout.size()) as *mut u8;
        }

        // Because accessing PROTECTION_LEVEL has the potential to trigger an allocation, 
        // we need to spin until we can claim the INTERNAL_ALLOCATION lock for our thread.
        claim_internal_alloc();
        let protection_level: Result<u32, ()> = PROTECTION_LEVEL.try_with(|v| *v.read()).or(Ok(0));
        release_internal_alloc();

        match protection_level {
            Ok(v) if v == 0 => malloc(layout.size()) as *mut u8,
            Ok(v) => {
                // Tripped a bad allocation, but make sure further allocation/deallocation during unwind
                // doesn't have issues
                PROTECTION_LEVEL.with(|v| *v.write() = 0);
                panic!("Unexpected allocation for size {}, protection level: {}", layout.size(), v)
            },
            Err(_) => {
                // Shouldn't be possible to get here
                panic!("Unexpected error checking protection level")
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if alloc_immediate() {
            return free(ptr as *mut c_void);
        }

        claim_internal_alloc();
        let protection_level: Result<u32, ()> = PROTECTION_LEVEL.try_with(|v| *v.read()).or(Ok(0));
        release_internal_alloc();

        free(ptr as *mut c_void);
        match protection_level {
            Ok(v) if v > 0 => {
                // Tripped a bad dealloc, but make sure further memory access during unwind
                // doesn't have issues
                PROTECTION_LEVEL.with(|v| *v.write() = 0);
                panic!("Unexpected deallocation for size {}, protection level: {}", layout.size(), v)
            },
            _ => ()
        }
    }
}
