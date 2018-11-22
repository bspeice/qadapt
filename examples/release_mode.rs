extern crate qadapt;

use qadapt::allocate_panic;
use qadapt::QADAPT;

#[global_allocator]
static Q: QADAPT = QADAPT;

#[allocate_panic]
fn does_allocate() -> Box<u8> {
    Box::new(0)
}

fn main() {
    // If you were to run `cargo run --example release_mode`, this program blows up.
    // If, however, you ran `cargo run --release --example release_mode`,
    // nothing interesting will happen since panic-related code is stripped
    // for release builds.
    does_allocate();
}
