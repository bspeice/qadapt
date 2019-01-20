use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[cfg(not(debug_assertions))]
#[test]
fn release_only_inactive() {
    assert!(!qadapt::is_active());
}
