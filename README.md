The Quick And Dirty Allocation Profiling Tool
=============================================

[![crates.io](https://img.shields.io/crates/v/qadapt.svg)](https://crates.io/crates/qadapt)
[![docs.rs](https://docs.rs/qadapt/badge.svg)](https://docs.rs/qadapt/)

This allocator is a helper for writing high-performance code that is allocation/drop free;
for functions annotated with `#[allocate_panic]`, QADAPT will detect when allocations/drops
happen during their execution (and execution of any functions they call) and throw a
thread panic if this occurs.

Because QADAPT panics on allocation and is rather slow (for an allocator) it is **strongly**
recommended that QADAPT (the allocator) be used only in code tests. Functions annotated with
`#[allocate_panic]` will have no side effects if the QADAPT allocator is not being used,
so the attribute is safe to leave everywhere.

Currently this crate is Nightly-only, but will work once `const fn` is in Stable.