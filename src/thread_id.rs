/// Taken from https://crates.io/crates/thread-id and re-purposed to be no-std safe
use libc;

pub fn get() -> usize {
    unsafe { libc::pthread_self() as usize }
}