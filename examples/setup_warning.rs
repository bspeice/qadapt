use env_logger;


use qadapt::allocate_panic;

// Note that we're missing the `#[global_allocator]` attribute

#[allocate_panic]
fn does_allocate() -> Box<u8> {
    Box::new(0)
}

fn main() {
    // This code will warn that QADAPT isn't being used, but won't trigger a panic.
    // Run with `RUST_LOG=warn cargo run --example setup_warning`
    env_logger::init();
    does_allocate();

    // The warning will only trigger once though
    does_allocate();
}
