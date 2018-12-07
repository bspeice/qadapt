# qadapt

[![crates.io](https://img.shields.io/crates/v/qadapt.svg)](https://crates.io/crates/qadapt)
[![docs.rs](https://docs.rs/qadapt/badge.svg)](https://docs.rs/qadapt/)
[![codecov](https://codecov.io/gh/bspeice/qadapt/branch/master/graph/badge.svg)](https://codecov.io/gh/bspeice/qadapt)
[![travisci](https://travis-ci.org/bspeice/qadapt.svg?branch=master)](https://travis-ci.org/bspeice/qadapt)
[![appveyor](https://ci.appveyor.com/api/projects/status/km1p081tkjcptn1w/branch/master?svg=true)](https://ci.appveyor.com/project/bspeice/qadapt/branch/master)


---
# QADAPT - `debug_assert!` for your memory

This allocator is a helper for writing high-performance code that is memory-sensitive;
a thread panic will be triggered if a function annotated with `#[no_alloc]`,
or code inside an `assert_no_alloc!` macro interacts with the allocator in any way.
Wanton allocations and unforeseen drops no more - this library lets you focus on
writing code without worrying if Rust properly managed to inline the variable into the stack.

Now, an allocator blowing up in production is a scary thought; that's why QADAPT
is designed to strip its own code out whenever you're running with a release build.
Just like the [`debug_assert!` macro](https://doc.rust-lang.org/std/macro.debug_assert.html)
in Rust's standard library, it's safe to use without worrying about a unforeseen
circumstance causing your application to crash.

# Usage

Actually making use of QADAPT is straight-forward. To set up the allocator,
place the following snippet in either your program binaries (main.rs) or tests:

```rust,ignore
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;
```

After that, there are two ways of telling QADAPT that it should trigger a panic:

1. Annotate functions with the `#[no_alloc]` proc macro:
```rust,no_run
use qadapt::no_alloc;

#[no_alloc]
fn do_math() -> u8 {
2 + 2
}
```

2. Evaluate expressions with the `assert_no_alloc!` macro
```rust,no_run
use qadapt::assert_no_alloc;

fn do_work() {
// This code is allowed to trigger an allocation
let b = Box::new(8);

// This code would panic if an allocation occurred inside it
let x = assert_no_alloc!(*b + 2);
assert_eq!(x, 10);
}
