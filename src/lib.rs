//! # QADAPT - `debug_assert!` for your memory
//!
//! This allocator is a helper for writing high-performance code that is memory-sensitive;
//! a thread panic will be triggered if a function annotated with `#[no_alloc]`,
//! or code inside an `assert_no_alloc!` macro interacts with the allocator in any way.
//! Wanton allocations and unforeseen drops no more - this library lets you focus on
//! writing code without worrying if Rust properly managed to inline the variable into the stack.
//! 
//! Now, an allocator blowing up in production is a scary thought; that's why QADAPT
//! is designed to strip its own code out whenever you're running with a release build.
//! Just like the [`debug_assert!` macro](https://doc.rust-lang.org/std/macro.debug_assert.html)
//! in Rust's standard library, it's safe to use without worrying about a unforeseen
//! circumstance causing your application to crash.
//! 
//! # Usage
//! 
//! Actually making use of QADAPT is straight-forward. To set up the allocator,
//! place the following snippet in either your program binaries (main.rs) or tests:
//! 
//! ```rust,ignore
//! use qadapt::QADAPT;
//! 
//! #[global_allocator]
//! static Q: QADAPT = QADAPT;
//! ```
//! 
//! After that, there are two ways of telling QADAPT that it should trigger a panic:
//! 
//! 1. Annotate functions with the `#[no_alloc]` proc macro:
//! ```rust,no_run
//! use qadapt::no_alloc;
//! 
//! #[no_alloc]
//! fn do_math() -> u8 {
//!     2 + 2
//! }
//! ```
//! 
//! 2. Evaluate expressions with the `assert_no_alloc!` macro
//! ```rust,no_run
//! use qadapt::assert_no_alloc;
//! 
//! fn do_work() {
//!     // This code is allowed to trigger an allocation
//!     let b = Box::new(8);
//!     
//!     // This code would panic if an allocation occurred inside it
//!     let x = assert_no_alloc!(*b + 2);
//!     assert_eq!(x, 10);
//! }
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
            panic!("QADAPT not initialized when using allocation guards; please verify `#[global_allocator]` is set!");
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

/// Get the result of an expression, guaranteeing that no memory accesses occur
/// during its evaluation.
/// 
/// **Warning**: Unexpected behavior may occur when using the `return` keyword.
/// Because the macro cleanup logic will not be run, QADAPT may trigger a panic
/// in code that was not specifically intended to be allocation-free.
#[macro_export]
macro_rules! assert_no_alloc {
    ($e:expr) => {{
        ::qadapt::enter_protected();
        let e = { $e };
        ::qadapt::exit_protected();
        e
    }};
}

static IS_ACTIVE: RwLock<bool> = RwLock::new(false);
static INTERNAL_ALLOCATION: RwLock<usize> = RwLock::new(usize::max_value());

/// Get the current "protection level" in QADAPT: calls to enter_protected() - exit_protected()
pub fn protection_level() -> usize {
    if cfg!(debug_assertions) {
        PROTECTION_LEVEL.try_with(|v| *v.read()).unwrap_or(0)
    } else {
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
        // we need to acquire the INTERNAL_ALLOCATION lock for our thread.
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
                // Tripped a bad drop, but make sure further memory access during unwind
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
