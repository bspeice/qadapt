# qadapt

[![crates.io](https://img.shields.io/crates/v/qadapt.svg)](https://crates.io/crates/qadapt)
[![docs.rs](https://docs.rs/qadapt/badge.svg)](https://docs.rs/qadapt/)
[![travisci](https://travis-ci.org/bspeice/qadapt.svg?branch=master)](https://travis-ci.org/bspeice/qadapt)
<!--
    AppVeyor badges use a unique ID that we're not able to compute ahead of time.
    Please see https://ci.appveyor.com/project/bspeice/qadapt/settings/badges
    to set up the badge
-->
[![codecov](https://codecov.io/gh/bspeice/qadapt/branch/master/graph/badge.svg)](https://codecov.io/gh/bspeice/qadapt)

---
# The Quick And Dirty Allocation Profiling Tool

This allocator is a helper for writing high-performance code that is allocation/drop free;
for functions annotated with `#[allocate_panic]`, QADAPT will detect when allocations/drops
happen during their execution (and execution of any functions they call) and throw a
thread panic if this occurs. QADAPT-related code is *stripped out during release builds*,
so no worries about random allocations crashing in production.

Currently this crate is Nightly-only, but will work once `const fn` is in Stable.

Please also take a look at [qadapt-macro](https://github.com/bspeice/qadapt/tree/master/qadapt-macro)
for some helper macros to make working with QADAPT a bit easier.
