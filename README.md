The Quick And Dirty Allocation Profiling Tool
=============================================

A simple attempt at an allocator that can let you know if allocations
are happening in places you didn't intend. This is primarily used for
guaranteeing that performance-critical code doesn't trigger an allocation
while running.
