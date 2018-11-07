extern crate backtrace;
extern crate libc;
extern crate qadapt_macro;
#[macro_use]
extern crate log;
extern crate spin;

// Re-export the proc macros to use by other code
pub use qadapt_macro::*;

use backtrace::Backtrace;
use libc::c_void;
use libc::free;
use libc::malloc;
use log::Level;
use std::alloc::Layout;
use std::alloc::GlobalAlloc;
use spin::RwLock;

static DO_PANIC: RwLock<bool> = RwLock::new(false);
static INTERNAL_ALLOCATION: RwLock<bool> = RwLock::new(false);
static LOG_LEVEL: RwLock<Level> = RwLock::new(Level::Debug);

pub struct QADAPT;

pub fn set_panic(b: bool) {
    let mut val = DO_PANIC.write();
    if *val == b {
        let level = LOG_LEVEL.read();
        if log_enabled!(*level) {
            log!(*level, "Panic flag was already {}, potential data race", b)
        }
    }

    *val = b;
}

pub fn set_log_level(level: Level) {
    *LOG_LEVEL.write() = level;
}

unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Use a block to release the read guard
        let should_panic = { *DO_PANIC.read() };

        if should_panic && !*INTERNAL_ALLOCATION.read() {
            // Only trip one panic at a time, don't want to cause issues on potential rewind
            *DO_PANIC.write() = false;
            panic!("Unexpected allocation")
        } else if log_enabled!(*LOG_LEVEL.read()) {
            // We wrap in a block because we need to release the write guard
            // so allocations during `Backtrace::new()` can read
            { *INTERNAL_ALLOCATION.write() = true; }

            let bt = Backtrace::new();
            log!(*LOG_LEVEL.read(), "Unexpected allocation:\n{:?}", bt);

            *INTERNAL_ALLOCATION.write() = false;
        }

        malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if *DO_PANIC.read() && !*INTERNAL_ALLOCATION.read() {
            panic!("Unexpected drop")
        } else if log_enabled!(*LOG_LEVEL.read()) {
            // We wrap in a block because we need to release the write guard
            // so allocations during `Backtrace::new()` can read
            { *INTERNAL_ALLOCATION.write() = true; }

            let bt = Backtrace::new();
            log!(*LOG_LEVEL.read(), "Unexpected drop:\n{:?}", bt);

            *INTERNAL_ALLOCATION.write() = false;
        }
        free(ptr as *mut c_void)
    }
}
