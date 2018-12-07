//! # The Quick And Dirty Allocation Profiling Tool
//!
//! This allocator is a helper for writing high-performance code that is allocation/drop free;
//! for functions annotated with `#[allocate_panic]`, QADAPT will detect when allocations/drops
//! happen during their execution (and execution of any functions they call) and throw a
//! thread panic if this occurs. QADAPT-related code is *stripped out during release builds*,
//! so no worries about random allocations crashing in production.
//!
//! Currently this crate is Nightly-only, but will work once `const fn` is in Stable.
//!
//! Please also take a look at [qadapt-macro](https://github.com/bspeice/qadapt/tree/master/qadapt-macro)
//! for some helper macros to make working with QADAPT a bit easier.
#![deny(missing_docs)]

use log::warn;
// thread_id is necessary because `std::thread::current()` panics if we have not yet
// allocated a `thread_local!{}` it depends on.
use thread_id;

// Re-export the proc macros to use by other code
pub use qadapt_macro::*;

use libc::c_void;
use libc::free;
use libc::malloc;
use spin::RwLock;
use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::thread;

thread_local! {
    static PROTECTION_LEVEL: RwLock<usize> = RwLock::new(0);
}

/// The QADAPT allocator itself
pub struct QADAPT;

/// Let QADAPT know that we are now entering a protected region and that
/// panics should be triggered if allocations/drops happen while we are running.
pub fn enter_protected() {
    #[cfg(debug_assertions)]
    {
        if thread::panicking() {
            return;
        }

        if !*IS_ACTIVE.read() {
            *IS_ACTIVE.write() = true;
            warn!("QADAPT not initialized when using allocation guards; please verify `#[global_allocator]` is set!");
        }

        PROTECTION_LEVEL
            .try_with(|v| {
                *v.write() += 1;
            })
            .unwrap_or_else(|_e| ());
    }
}

/// Let QADAPT know that we are exiting a protected region. Will panic
/// if we attempt to [`exit_protected`] more times than we [`enter_protected`].
pub fn exit_protected() {
    #[cfg(debug_assertions)]
    {
        if thread::panicking() {
            return;
        }

        PROTECTION_LEVEL
            .try_with(|v| {
                let val = { *v.read() };
                match val {
                    v if v == 0 => panic!("Attempt to exit protected too many times"),
                    _ => {
                        *v.write() -= 1;
                    }
                }
            })
            .unwrap_or_else(|_e| ());
    }
}

static IS_ACTIVE: RwLock<bool> = RwLock::new(false);
static INTERNAL_ALLOCATION: RwLock<usize> = RwLock::new(usize::max_value());

/// Get the current "protection level" in QADAPT: calls to enter_protected() - exit_protected()
pub fn protection_level() -> usize {
    #[cfg(debug_assertions)]
    {
        PROTECTION_LEVEL.try_with(|v| *v.read()).unwrap_or(0)
    }
    #[cfg(not(debug_assertions))]
    {
        0
    }
}

fn claim_internal_alloc() {
    loop {
        match INTERNAL_ALLOCATION.write() {
            ref mut lock if **lock == usize::max_value() => {
                **lock = thread_id::get();
                break;
            }
            _ => (),
        }
    }
}

fn release_internal_alloc() {
    match INTERNAL_ALLOCATION.write() {
        ref mut lock if **lock == thread_id::get() => **lock = usize::max_value(),
        _ => panic!("Internal allocation tracking error"),
    }
}

fn alloc_immediate() -> bool {
    thread::panicking() || *INTERNAL_ALLOCATION.read() == thread_id::get()
}

unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !*IS_ACTIVE.read() {
            *IS_ACTIVE.write() = true;
        }

        // If we're attempting to allocate our PROTECTION_LEVEL thread local,
        // just allow it through
        if alloc_immediate() {
            return malloc(layout.size()) as *mut u8;
        }

        // Because accessing PROTECTION_LEVEL has the potential to trigger an allocation,
        // we need to spin until we can claim the INTERNAL_ALLOCATION lock for our thread.
        claim_internal_alloc();
        let protection_level: Result<usize, ()> =
            PROTECTION_LEVEL.try_with(|v| *v.read()).or(Ok(0));
        release_internal_alloc();

        match protection_level {
            Ok(v) if v == 0 => malloc(layout.size()) as *mut u8,
            Ok(v) => {
                // Tripped a bad allocation, but make sure further memory access during unwind
                // doesn't have issues
                PROTECTION_LEVEL.with(|v| *v.write() = 0);
                panic!(
                    "Unexpected allocation for size {}, protection level: {}",
                    layout.size(),
                    v
                )
            }
            Err(_) => unreachable!(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if alloc_immediate() {
            return free(ptr as *mut c_void);
        }

        claim_internal_alloc();
        let protection_level: Result<usize, ()> =
            PROTECTION_LEVEL.try_with(|v| *v.read()).or(Ok(0));
        release_internal_alloc();

        // Free before checking panic to make sure we avoid leaks
        free(ptr as *mut c_void);
        match protection_level {
            Ok(v) if v > 0 => {
                // Tripped a bad dealloc, but make sure further memory access during unwind
                // doesn't have issues
                PROTECTION_LEVEL.with(|v| *v.write() = 0);
                panic!(
                    "Unexpected deallocation for size {}, protection level: {}",
                    layout.size(),
                    v
                )
            }
            _ => (),
        }
    }
}
