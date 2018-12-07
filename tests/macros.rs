use std::io;

use qadapt::allocate_panic;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[allocate_panic]
fn no_allocate() {
    let _v: Vec<()> = Vec::with_capacity(0);
}

#[test]
fn macro_no_allocate() {
    no_allocate();
}

#[allocate_panic]
fn allocates() {
    assert_eq!(::qadapt::protection_level(), 1);
    // Without boxing, release profile can actually optimize out the allocation
    let mut v = Box::new(Vec::new());
    v.push(1);
}

#[test]
#[should_panic]
fn macro_allocates() {
    allocates();
}

#[allocate_panic]
fn no_allocate_ret() -> bool {
    return true;
}

#[test]
fn macro_return() {
    assert!(no_allocate_ret());
}

#[allocate_panic]
fn no_allocate_implicit_ret() -> bool {
    true
}

#[test]
fn macro_implicit_return() {
    assert!(no_allocate_implicit_ret());
}

#[allocate_panic]
fn no_allocate_arg(b: bool) -> bool {
    b
}

#[test]
fn macro_allocate_arg() {
    no_allocate_arg(true);
    no_allocate_arg(false);
}

#[allocate_panic]
fn no_allocate_args(_b: bool, _u: usize, i: i64) -> i64 {
    i
}

#[test]
fn macro_allocate_args() {
    no_allocate_args(true, 0, -1);
    no_allocate_args(false, 4, -90);
}

#[allocate_panic]
fn return_result(r: Result<usize, io::Error>) -> Result<Result<usize, io::Error>, ()> {
    Ok(r)
}

#[test]
fn macro_return_result() {
    return_result(Ok(16)).unwrap().unwrap();
}

#[allocate_panic]
fn branching_return(a: bool, b: bool, c: bool) -> u8 {
    if a {
        if b {
            if c {
                return 1;
            } else {
                return 2;
            }
        } else {
            if c {
                return 3;
            } else {
                return 4;
            }
        }
    } else {
        if b {
            if c {
                return 5;
            } else {
                return 6;
            }
        } else {
            if c {
                return 7;
            } else {
                return 8;
            }
        }
    }
}

#[test]
fn macro_branching_return() {
    assert_eq!(1, branching_return(true, true, true));
    assert_eq!(2, branching_return(true, true, false));
    assert_eq!(3, branching_return(true, false, true));
    assert_eq!(4, branching_return(true, false, false));
    assert_eq!(5, branching_return(false, true, true));
    assert_eq!(6, branching_return(false, true, false));
    assert_eq!(7, branching_return(false, false, true));
    assert_eq!(8, branching_return(false, false, false));
}

fn run_closure(x: impl Fn(bool, bool) -> bool) -> bool {
    x(true, false)
}

#[allocate_panic]
fn example_closure() {
    let c = run_closure(|a: bool, b| return a && b);
    assert!(!c);
    let x = || return true;
    assert!(x());
}

#[test]
fn macro_closure() {
    example_closure()
}

#[test]
#[allocate_panic]
fn macro_release_safe() {
    #[cfg(debug_assertions)]
    {
        assert_eq!(1, ::qadapt::protection_level());
    }
    #[cfg(not(debug_assertions))]
    {
        assert_eq!(0, ::qadapt::protection_level());
    }
}
