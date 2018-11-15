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

fn release_guard(fn_name: &str) -> TokenStream {
    let rel = I::new("release", Span::call_site());
    let not_rel: Vec<TokenTree> = vec![
        I::new("not", Span::call_site()).into(),
        G::new(Delimiter::Parenthesis, TokenTree::Ident(rel).into()).into()
    ];
    let cfg_not_rel: Vec<TokenTree> = vec![
        I::new("cfg", Span::call_site()).into(),
        G::new(Delimiter::Parenthesis, TS::from_iter(not_rel.into_iter())).into()
    ];
    let guarded: Vec<TokenTree> = vec![
        P::new('#', Spacing::Alone).into(),
        G::new(Delimiter::Bracket, TS::from_iter(cfg_not_rel.into_iter())).into(),
        P::new(':', Spacing::Joint).into(),
        P::new(':', Spacing::Alone).into(),
        I::new("qadapt", Span::call_site()).into(),
        P::new(':', Spacing::Joint).into(),
        P::new(':', Spacing::Alone).into(),
        I::new(fn_name, Span::call_site()).into(),
        G::new(Delimiter::Parenthesis, TokenStream::new()).into(),
    ];

    TS::from_iter(guarded.into_iter())
}

fn protected_body(fn_name: &str, args: G) -> TokenTree {
    let mut args_filtered = Vec::new();
    let mut waiting_for_comma = false;
    let mut in_type = 0;
    for tt in args.stream().into_iter() {
        match tt {
            TokenTree::Ident(ref _i) if !waiting_for_comma && in_type == 0 => {
                args_filtered.push(tt.clone());
                waiting_for_comma = true;
            }
            TokenTree::Punct(ref p) if p.as_char() == '<' => in_type += 1,
            TokenTree::Punct(ref p) if p.as_char() == '>' => in_type -= 1,
            TokenTree::Punct(ref p) if p.as_char() == ',' && in_type == 0 => {
                waiting_for_comma = false;
                args_filtered.push(tt.clone())
            }
            _ => ()
        }
    }
    let args_group = G::new(Delimiter::Parenthesis, TS::from_iter(args_filtered));

    let tt: Vec<TT> = vec![
        G::new(Delimiter::Brace, release_guard("enter_protected")).into(),
        I::new("let", Span::call_site()).into(),
        I::new("__ret__", Span::call_site()).into(),
        P::new('=', Spacing::Alone).into(),
        I::new(fn_name, Span::call_site()).into(),
        args_group.into(),
        P::new(';', Spacing::Alone).into(),
        G::new(Delimiter::Brace, release_guard("exit_protected")).into(),
        I::new("__ret__", Span::call_site()).into(),
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
    let mut original_fn: Vec<TokenTree> = Vec::new();
    let mut protected_fn: Vec<TokenTree> = Vec::new();

    let mut item_iter = item.into_iter();

    // First, get the function name we're replacing
    let mut fn_name = None;
    let mut fn_args = None;
    while let Some(tt) = item_iter.next() {
        match tt {
            TokenTree::Ident(ref i) if i.to_string() == "fn" => {
                original_fn.push(tt.clone());
                protected_fn.push(tt.clone());
            }
            TokenTree::Ident(ref i) if fn_args.is_none() => {
                let changed_name = format!("__{}__", i.to_owned());
                original_fn.push(TokenTree::Ident(I::new(&changed_name, Span::call_site())));
                protected_fn.push(tt.clone());
                fn_name = Some(changed_name);
            }
            TokenTree::Group(ref g) if g.delimiter() == Delimiter::Parenthesis && fn_args.is_none() => {
                original_fn.push(tt.clone());
                protected_fn.push(tt.clone());
                fn_args = Some(g.clone());
            }
            TokenTree::Group(ref g) if g.delimiter() == Delimiter::Brace => {
                original_fn.push(tt.clone());
                protected_fn.push(protected_body(
                    &fn_name.take().unwrap(),
                    fn_args.take().unwrap(),
                ));
            }
            tt => {
                original_fn.push(tt.clone());
                protected_fn.push(tt.clone());
            }
        }
    }

    let mut full = Vec::new();
    full.push(TS::from_iter(original_fn));
    full.push(TS::from_iter(protected_fn));
    let ts = TS::from_iter(full.into_iter());
    ts
}
