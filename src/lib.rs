//! ## `debug_assert!` for your memory usage
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
//! ```rust
//! use qadapt::QADAPT;
//!
//! #[global_allocator]
//! static Q: QADAPT = QADAPT;
//!
//! fn main() {
//!     # // Because `debug_assertions` are on for doctests in release mode
//!     # // we have to add an extra guard.
//!     # if qadapt::is_active() {
//!     if cfg!(debug_assertions) {
//!         assert!(qadapt::is_active());
//!     }
//!     # }
//! }
//! ```
//!
//! After that, there are two ways of telling QADAPT that it should trigger a panic:
//!
//! 1. Annotate functions with the `#[no_alloc]` proc macro:
//! ```rust
//! use qadapt::no_alloc;
//! use qadapt::QADAPT;
//! use std::panic::catch_unwind;
//!
//! #[global_allocator]
//! static Q: QADAPT = QADAPT;
//!
//! // This function is fine, there are no allocations here
//! #[no_alloc]
//! fn do_math() -> u8 {
//!     2 + 2
//! }
//!
//! // This function will trigger a panic when called
//! #[no_alloc]
//! fn does_panic() -> Box<u32> {
//!     Box::new(5)
//! }
//!
//! fn main() {
//!     do_math();
//!
//!     let err = catch_unwind(|| does_panic());
//!     # if qadapt::is_active() {
//!     assert!(err.is_err());
//!     # }
//! }
//! ```
//!
//! 2. Evaluate expressions with the `assert_no_alloc!` macro
//! ```rust
//! use qadapt::assert_no_alloc;
//! use qadapt::QADAPT;
//!
//! #[global_allocator]
//! static Q: QADAPT = QADAPT;
//!
//! fn main() {
//!     // This code is allowed to trigger an allocation
//!     let b = Box::new(8);
//!     
//!     // This code would panic if an allocation occurred inside it
//!     let x = assert_no_alloc!(*b + 2);
//!     assert_eq!(x, 10);
//! }
//! ```
#![deny(missing_docs)]

// thread_id is necessary because `std::thread::current()` panics if we have not yet
// allocated a `thread_local!{}` it depends on.
use thread_id;

// Re-export the proc macros to use by other code
pub use qadapt_macro::*;

use spin::RwLock;
use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::thread;

thread_local! {
    static PROTECTION_LEVEL: RwLock<usize> = RwLock::new(0);
}
static IS_ACTIVE: RwLock<bool> = RwLock::new(false);
static INTERNAL_ALLOCATION: RwLock<usize> = RwLock::new(usize::max_value());


/// The QADAPT allocator itself
///
/// To make use of the allocator, include this code block in your program
/// binaries/tests:
///
/// ```rust
/// use qadapt::QADAPT;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// fn main() {
///     # if qadapt::is_active() {
///     if cfg!(debug_assertions) {
///         assert!(qadapt::is_active());
///     }
///     # }
/// }
/// ```
pub struct QADAPT;

static SYSTEM_ALLOC: System = System;

/// Let QADAPT know that we are now entering a protected region and that
/// panics should be triggered if allocations/drops happen while we are running.
///
/// **Example**:
///
/// ```rust
/// use qadapt::enter_protected;
/// use qadapt::exit_protected;
/// use qadapt::QADAPT;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// fn main() {
///     // Force an allocation by using a Box
///     let x = Box::new(2);
///
///     enter_protected();
///     // We're now in a memory-protected region - allocations and drops
///     // here will trigger thread panic
///     let y = *x * 4;
///     exit_protected();
///
///     // It's now safe to allocate/drop again
///     let z = Box::new(y);
/// }
pub fn enter_protected() {
    #[cfg(debug_assertions)]
    {
        if thread::panicking() || !is_active() {
            return;
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
///
/// **Example**:
///
/// ```rust
/// use qadapt::enter_protected;
/// use qadapt::exit_protected;
/// use qadapt::QADAPT;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// fn main() {
///     // Force an allocation by using a Box
///     let x = Box::new(2);
///
///     enter_protected();
///     // We're now in a memory-protected region - allocations and drops
///     // here will trigger thread panic
///     let y = *x * 4;
///     exit_protected();
///
///     // It's now safe to allocate/drop again
///     let z = Box::new(y);
/// }
pub fn exit_protected() {
    #[cfg(debug_assertions)]
    {
        if thread::panicking() || !is_active() {
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
/// **Example**:
///
/// ```rust
/// use qadapt::assert_no_alloc;
/// use qadapt::QADAPT;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// fn main() {
///     assert_no_alloc!(2 + 2);
/// }
/// ```
///
/// **Warning**: Unexpected behavior will occur when using the `return` keyword.
/// Because QADAPT doesn't have an opportunity to clean up, there may be a panic
/// in code that was not intended to be allocation-free. The compiler will warn you
/// that there is an unreachable statement if this happens.
///
/// ```rust
/// use qadapt::assert_no_alloc;
/// use qadapt::QADAPT;
/// use std::panic::catch_unwind;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// fn early_return() -> usize {
///     assert_no_alloc!(return 8);
/// }
///
/// fn main() {
///     let x = early_return();
///     
///     // Even though only the `early_return` function contains
///     // QADAPT allocation guards, this triggers a panic:
///     // `Box::new` forces an allocation, and QADAPT still thinks
///     // we're in a protected region because of the return in  `early_return()`
///     # if qadapt::is_active() {
///     let res = catch_unwind(|| Box::new(x));
///     assert!(res.is_err());
///     # }
/// }
#[macro_export]
macro_rules! assert_no_alloc {
    ($e:expr) => {{
        ::qadapt::enter_protected();
        let e = { $e };
        ::qadapt::exit_protected();
        e
    }};
}

/// Get the current "protection level" in QADAPT: calls to `enter_protected() - exit_protected()`.
///
/// **Note**: For release builds, `protection_level()` will always return 0.
///
/// **Example**:
///
/// ```rust
/// use qadapt::enter_protected;
/// use qadapt::exit_protected;
/// use qadapt::QADAPT;
/// use qadapt::protection_level;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// fn main() {
///     # if qadapt::is_active() {
///     enter_protected();
///     // We're now in an allocation-protected code region
///     assert_eq!(1, protection_level());
///     
///     enter_protected();
///     // We're still memory protected, but we'll now need to exit twice to be safe
///     assert_eq!(2, protection_level());
///     exit_protected();
///     exit_protected();
///     # }
///     
///     // It's now safe to allocate/drop
/// }
pub fn protection_level() -> usize {
    PROTECTION_LEVEL.try_with(|v| *v.read()).unwrap_or(0)
}

/// Determine whether qadapt is running as the current global allocator. Useful for
/// double-checking that you will in fact panic if allocations happen in guarded code.
///
/// **Note**: when running in `release` profile, `is_active()` will always return false.
///
/// **Example**:
///
/// ```rust
/// use qadapt::is_active;
/// use qadapt::QADAPT;
///
/// #[global_allocator]
/// static Q: QADAPT = QADAPT;
///
/// pub fn main() {
///     # if qadapt::is_active() {
///     if cfg!(debug_assertions) {
///         assert!(is_active());
///     }
///     # }
/// }
/// ```
pub fn is_active() -> bool {
    if cfg!(debug_assertions) {
        *IS_ACTIVE.read()
    } else {
        false
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
            return SYSTEM_ALLOC.alloc(layout);
        }

        // Because accessing PROTECTION_LEVEL has the potential to trigger an allocation,
        // we need to acquire the INTERNAL_ALLOCATION lock for our thread.
        claim_internal_alloc();
        let protection_level: Result<usize, ()> =
            PROTECTION_LEVEL.try_with(|v| *v.read()).or(Ok(0));
        release_internal_alloc();

        match protection_level {
            Ok(v) if v == 0 => SYSTEM_ALLOC.alloc(layout),
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
            return SYSTEM_ALLOC.dealloc(ptr, layout);
        }

        claim_internal_alloc();
        let protection_level: Result<usize, ()> =
            PROTECTION_LEVEL.try_with(|v| *v.read()).or(Ok(0));
        release_internal_alloc();

        // Free before checking panic to make sure we avoid leaks
        SYSTEM_ALLOC.dealloc(ptr, layout);
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
