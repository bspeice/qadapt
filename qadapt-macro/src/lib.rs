//! Helper macros to use with the QADAPT allocator system
//! 
//! This crate is unusable on its own, but because `proc_macro` crates
//! can't export non-proc-macro symbols, we have to go through an extra step
//! to move these up.
//! 
//! Ultimately, this does actually work because we don't need to actually use
//! references to the underlying functionality here, we just need to know
//! where they will ultimately end up at.
// TODO: This causes issues, but I can't track down why
// #![deny(missing_docs)]
extern crate proc_macro;

use proc_macro::TokenTree;
use proc_macro::TokenStream;

/// Set up the QADAPT allocator to trigger a panic if any allocations happen during
/// this function. Race conditions between threads are not checked, this macro will
/// only work as intended in a single-threaded or otherwise synchronized environment.
#[proc_macro_attribute]
pub fn allocate_panic(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ret = item.clone();

    let mut token_iter = item.into_iter();
    match token_iter.next() {
        Some(TokenTree::Ident(ref i)) if i.to_string() == "fn" => (),
        _ => panic!("#[allocate_panic] macro can only be applied to functions")
    }

    ret
}
