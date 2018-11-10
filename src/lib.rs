extern crate libc;
extern crate qadapt_macro;
extern crate spin;

// Re-export the proc macros to use by other code
pub use qadapt_macro::*;

use libc::c_void;
use libc::free;
use libc::malloc;
use spin::Mutex;
use std::alloc::Layout;
use std::alloc::GlobalAlloc;
use std::sync::RwLock;
use std::thread;

static THREAD_LOCAL_LOCK: Mutex<()> = Mutex::new(());
thread_local! {
    static PROTECTION_LEVEL: RwLock<u32> = RwLock::new(0);
}

pub struct QADAPT;

pub fn enter_protected() {
    if thread::panicking() {
        return
    }

    PROTECTION_LEVEL.try_with(|v| {
        *v.write().unwrap() += 1;
    }).unwrap_or_else(|_e| ());
}

pub fn exit_protected() {
    if thread::panicking() {
        return
    }

    PROTECTION_LEVEL.try_with(|v| {
        let val = { *v.read().unwrap() };
        match val {
            v if v == 0 => panic!("Attempt to exit protected too many times"),
            _ => {
                *v.write().unwrap() -= 1;
            }
        }
    }).unwrap_or_else(|_e| ());
}

unsafe impl GlobalAlloc for QADAPT {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // If we're attempting to allocate our PROTECTION_LEVEL thread local,
        // just allow it through
        if thread::panicking() || THREAD_LOCAL_LOCK.try_lock().is_none() {
            return malloc(layout.size()) as *mut u8;
        }

        let protection_level: Result<u32, ()> = {
            let _lock = THREAD_LOCAL_LOCK.lock();
            PROTECTION_LEVEL.try_with(|v| *v.read().unwrap())
                .or(Ok(0))
        };

        match protection_level {
            Ok(v) if v == 0 => malloc(layout.size()) as *mut u8,
            //Ok(v) => panic!("Unexpected allocation for size {}, protection level: {}", layout.size(), v),
            Ok(v) => {
                // Tripped a bad allocation, but make sure further allocation/deallocation during unwind
                // doesn't have issues
                PROTECTION_LEVEL.with(|v| *v.write().unwrap() = 0);
                panic!("Unexpected allocation for size {}, protection level: {}", layout.size(), v)
            }
            Err(_) => {
                // It shouldn't be possible to reach this point...
                panic!("Unexpected error for fetching protection level")
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if thread::panicking() || THREAD_LOCAL_LOCK.try_lock().is_none() {
            return free(ptr as *mut c_void);
        }

        let protection_level: Result<u32, ()> = {
            let _lock = THREAD_LOCAL_LOCK.lock();
            PROTECTION_LEVEL.try_with(|v| *v.read().unwrap())
                .or(Ok(0))
        };

        free(ptr as *mut c_void);
        match protection_level {
            Ok(v) if v > 0 => {
                // Tripped a bad dealloc, but make sure further memory access during unwind
                // doesn't have issues
                PROTECTION_LEVEL.with(|v| *v.write().unwrap() = 0);
                panic!("Unexpected deallocation for size {}, protection level: {}", layout.size(), v)
            },
            _ => ()
        }
    }
}
