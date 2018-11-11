//! Helper macros to use with the QADAPT allocator system
//! 
//! This crate is intended for managing the QADAPT allocator,
//! and is unusable on its own.
//! 
// TODO: This causes issues, but I can't track down why
// #![deny(missing_docs)]
extern crate proc_macro;

use proc_macro::TokenTree;
use proc_macro::TokenStream;

/// Set up the QADAPT allocator to trigger a panic if any allocations happen during
/// calls to this function.
/// 
/// QADAPT will only track allocations in the thread that calls this function;
/// if (for example) this function receives the results of an allocation in a
/// separate thread, QADAPT will not trigger a panic.
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
