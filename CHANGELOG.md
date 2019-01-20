# Version 1.1.0

- Don't panic if calling guarded code and QADAPT isn't the allocator;
  Instead, let code determine at runtime whether or not QADAPT is enabled
  and let users build their own asserts - #7
  - This fixes issues where libraries making use of QADAPT would trigger
    panics in anyone that didn't use the library.


# Version 1.0.1 (2019-01-01)

- Use the system allocator and remove libc dependency

# Version 1.0.0 (2018-12-15)

- Now working on Stable because of the Rust 1.31 release (and `const fn`)
- Documentation added

# Version 0.7.0 (2018-12-03)

- Fix conditional compilation flags never stripping QADAPT code

# Version 0.6.0 (2018-11-17)

- Fixed exit early bugs when closures contained `return` statements
