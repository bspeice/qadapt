//! Helper macros to use with the QADAPT allocator system
//!
//! **Please note**: This crate has been deprecated in favor of [alloc-counter](https://crates.io/crates/alloc_counter)
//!
//! This crate is intended for managing the QADAPT allocator,
//! and is unusable on its own.
//!
// TODO: This causes issues, but I can't track down why
// #![deny(missing_docs)]
extern crate proc_macro;

use proc_macro::Delimiter;
use proc_macro::Group;
use proc_macro::Spacing;
use proc_macro::TokenStream;
use proc_macro::TokenTree;
use std::iter::FromIterator;

macro_rules! group {
    ($delim:expr, $ts:expr) => {{
        let _tt: TokenTree = ::proc_macro::Group::new($delim, $ts).into();
        _tt
    }};
    ($delim:expr) => {
        group!($delim, ::proc_macro::TokenStream::new())
    };
}

macro_rules! ident {
    ($name:expr, $span:expr) => {{
        let _tt: TokenTree = ::proc_macro::Ident::new($name, $span).into();
        _tt
    }};
    ($name:expr) => {
        ident!($name, ::proc_macro::Span::call_site())
    };
}

macro_rules! punct {
    ($ch:expr, $spacing:expr) => {{
        let _tt: TokenTree = ::proc_macro::Punct::new($ch, $spacing).into();
        _tt
    }};
}

macro_rules! token_stream {
    ($($tt:expr,)*) => {
        {
            let _v: Vec<::proc_macro::TokenTree> = vec![$($tt),*];
            let _ts: TokenStream = ::proc_macro::TokenStream::from_iter(_v.into_iter());
            _ts
        }
    };
    ($($tt:expr),*) => {
        {
            let _v: Vec<::proc_macro::TokenTree> = vec![$($tt),*];
            let _ts: TokenStream = ::proc_macro::TokenStream::from_iter(_v.into_iter());
            _ts
        }
    };
}

/// Generate the body of a function that is protected from making allocations.
/// The code is conditionally compiled so that all QADAPT-related bits
/// will be removed for release/bench builds, making the proc_macro safe
/// to leave on in production.
#[rustfmt::skip]
fn protected_body(fn_body: Group) -> TokenTree {
    group!(Delimiter::Brace, token_stream!(
        group!(Delimiter::Brace, token_stream!(
            punct!(':', Spacing::Joint),
            punct!(':', Spacing::Alone),
            ident!("qadapt"),
            punct!(':', Spacing::Joint),
            punct!(':', Spacing::Alone),
            ident!("enter_protected"),
            group!(Delimiter::Parenthesis)
        )),
        ident!("let"),
        ident!("__ret__"),
        punct!('=', Spacing::Alone),
        fn_body.into(),
        punct!(';', Spacing::Alone),
        punct!('#', Spacing::Alone),
        // When `return` statements are involved, this code can get marked as
        // unreachable because of early exit
        group!(Delimiter::Bracket, token_stream!(
            ident!("allow"),
            group!(Delimiter::Parenthesis, token_stream!(
                ident!("unreachable_code")
            ))
        )),
        group!(Delimiter::Brace, token_stream!(
            group!(Delimiter::Brace, token_stream!(
                punct!(':', Spacing::Joint),
                punct!(':', Spacing::Alone),
                ident!("qadapt"),
                punct!(':', Spacing::Joint),
                punct!(':', Spacing::Alone),
                ident!("exit_protected"),
                group!(Delimiter::Parenthesis)
            )),
            ident!("__ret__")
        ))
    ))
}

/// Walk through a TokenStream (typically from a Group Brace) and prepend calls
/// to `return` with an exit guard.
fn escape_return(ts: TokenStream) -> TokenStream {
    let mut protected: Vec<TokenTree> = Vec::new();
    let mut in_closure: bool = false;

    let mut tt_iter = ts.into_iter();
    while let Some(tt) = tt_iter.next() {
        let tokens = match tt {
            TokenTree::Group(ref g) if g.delimiter() == Delimiter::Brace && !in_closure => {
                vec![group!(Delimiter::Brace, escape_return(g.stream()))]
            }
            TokenTree::Ident(ref i) if i.to_string() == "return" && !in_closure => vec![
                group!(
                    Delimiter::Brace,
                    token_stream!(
                        punct!(':', Spacing::Joint),
                        punct!(':', Spacing::Alone),
                        ident!("qadapt"),
                        punct!(':', Spacing::Joint),
                        punct!(':', Spacing::Alone),
                        ident!("exit_protected"),
                        group!(Delimiter::Parenthesis)
                    )
                ),
                tt.clone(),
            ],
            TokenTree::Punct(ref p) if p.as_char() == '|' => {
                in_closure = true;
                vec![tt.clone()]
            }
            TokenTree::Punct(ref p) if p.as_char() == ';' => {
                in_closure = false;
                vec![tt.clone()]
            }
            t => vec![t],
        };

        protected.extend(tokens.into_iter());
    }

    TokenStream::from_iter(protected.into_iter())
}

/// Set up the QADAPT allocator to trigger a panic if any allocations happen during
/// calls to this function.
///
/// QADAPT will only track allocations in the current function call;
/// if (for example) this function receives the results of an allocation in a
/// separate thread, or defers allocations via closure/Future, those results
/// will not trigger an error.
#[proc_macro_attribute]
#[deprecated(
    since = "1.0.3",
    note = "Please use the `alloc_counter` crate instead."
)]
pub fn no_alloc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut protected_fn: Vec<TokenTree> = Vec::new();
    let mut item_iter = item.into_iter();

    // First, get the function body we're replicating
    let mut fn_body = None;
    while let Some(tt) = item_iter.next() {
        match tt {
            TokenTree::Group(ref g) if g.delimiter() == Delimiter::Brace => {
                fn_body = Some(Group::new(Delimiter::Brace, escape_return(g.stream())));
                break;
            }
            tt => {
                protected_fn.push(tt.clone());
            }
        }
    }

    protected_fn.push(protected_body(fn_body.as_ref().unwrap().clone()));

    while let Some(tt) = item_iter.next() {
        protected_fn.push(tt)
    }

    TokenStream::from_iter(protected_fn.into_iter())
}
