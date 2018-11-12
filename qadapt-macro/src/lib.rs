//! Helper macros to use with the QADAPT allocator system
//!
//! This crate is intended for managing the QADAPT allocator,
//! and is unusable on its own.
//!
// TODO: This causes issues, but I can't track down why
// #![deny(missing_docs)]
extern crate proc_macro;

use proc_macro::Delimiter;
use proc_macro::Spacing;
use proc_macro::Span;
use proc_macro::TokenStream;
use proc_macro::TokenTree;
use std::iter::FromIterator;

type TT = proc_macro::TokenTree;
type TS = proc_macro::TokenStream;
type G = proc_macro::Group;
type I = proc_macro::Ident;
type P = proc_macro::Punct;

fn protected_group(fn_body: G) -> TokenTree {
    let tt: Vec<TT> = vec![
        P::new(':', Spacing::Joint).into(),
        P::new(':', Spacing::Alone).into(),
        I::new("qadapt", Span::call_site()).into(),
        P::new(':', Spacing::Joint).into(),
        P::new(':', Spacing::Alone).into(),
        I::new("enter_protected", Span::call_site()).into(),
        G::new(Delimiter::Parenthesis, TokenStream::new()).into(),
        P::new(';', Spacing::Alone).into(),
        fn_body.into(),
        P::new(':', Spacing::Joint).into(),
        P::new(':', Spacing::Alone).into(),
        I::new("qadapt", Span::call_site()).into(),
        P::new(':', Spacing::Joint).into(),
        P::new(':', Spacing::Alone).into(),
        I::new("exit_protected", Span::call_site()).into(),
        G::new(Delimiter::Parenthesis, TokenStream::new()).into(),
        P::new(';', Spacing::Alone).into(),
    ];

    G::new(Delimiter::Brace, TS::from_iter(tt)).into()
}

/// Set up the QADAPT allocator to trigger a panic if any allocations happen during
/// calls to this function.
///
/// QADAPT will only track allocations in the thread that calls this function;
/// if (for example) this function receives the results of an allocation in a
/// separate thread, QADAPT will not trigger a panic.
#[proc_macro_attribute]
pub fn allocate_panic(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ret: Vec<TokenTree> = Vec::new();

    let mut fn_body = None;
    let mut item_iter = item.into_iter();
    while let Some(tt) = item_iter.next() {
        match tt {
            TokenTree::Group(ref g) if g.delimiter() == Delimiter::Brace => {
                fn_body = Some(g.clone());
                break;
            }
            tt => ret.push(tt),
        }
    }
    ret.push(protected_group(fn_body.unwrap()));
    TokenStream::from_iter(ret.into_iter())
}
