The Quick And Dirty Allocation Profiling Tool
=============================================

A simple attempt at a `#[no_std]` compatible allocator that can track
allocations on a per-thread basis, for the purpose of guaranteeing that
performance-critical code doesn't trigger an allocation while running.

The current state has all the infrastructure in place, but the tests are a bit
flaky. As such, this crate likely won't see much further development; if you
are interested in claiming the qadapt name, please reach out to the author
at [bradlee@speice.io](mailto:bradlee@speice.io).
