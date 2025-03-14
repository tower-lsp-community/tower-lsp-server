//! Internal procedural macros for [`tower-lsp-server`](https://docs.rs/tower-lsp-server).
//!
//! This crate should not be used directly.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, ItemTrait, LitStr, ReturnType, TraitItem, Type};

/// Macro for generating LSP server implementation from [`lsp-types`](https://docs.rs/lsp-types).
///
/// This procedural macro annotates the `tower_lsp_server::LanguageServer` trait and generates a
/// corresponding `register_lsp_methods()` function which registers all the methods on that trait
/// as RPC handlers.
#[proc_macro_attribute]
pub fn rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Attribute will be parsed later in `parse_method_calls()`.
    if !attr.is_empty() {
        return item;
    }

    let mut lang_server_trait = parse_macro_input!(item as ItemTrait);
    let method_calls = parse_method_calls(&lang_server_trait);
    let req_types_and_router_fn = gen_server_router(&lang_server_trait.ident, &method_calls);
    require_methods_return_send_future(&mut lang_server_trait.items);

    let tokens = quote! {
        #lang_server_trait
        #req_types_and_router_fn
    };

    tokens.into()
}

struct MethodCall<'a> {
    rpc_name: String,
    handler_name: &'a syn::Ident,
}

fn parse_method_calls(lang_server_trait: &ItemTrait) -> Vec<MethodCall> {
    let mut calls = Vec::new();

    for item in &lang_server_trait.items {
        let method = match item {
            TraitItem::Fn(m) => m,
            _ => continue,
        };

        let attr = method
            .attrs
            .iter()
            .find(|attr| attr.meta.path().is_ident("rpc"))
            .expect("expected `#[rpc(name = \"foo\")]` attribute");

        let mut rpc_name = String::new();
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let s: LitStr = meta.value().and_then(|v| v.parse())?;
                rpc_name = s.value();
                Ok(())
            } else {
                Err(meta.error("expected `name` identifier in `#[rpc]`"))
            }
        })
        .unwrap();

        calls.push(MethodCall {
            rpc_name,
            handler_name: &method.sig.ident,
        });
    }

    calls
}

fn gen_server_router(trait_name: &syn::Ident, methods: &[MethodCall]) -> proc_macro2::TokenStream {
    let route_registrations: proc_macro2::TokenStream = methods
        .iter()
        .map(|method| {
            let rpc_name = &method.rpc_name;
            let handler = &method.handler_name;

            let layer = match &rpc_name[..] {
                "initialize" => quote! { layers::Initialize::new(state.clone(), pending.clone()) },
                "shutdown" => quote! { layers::Shutdown::new(state.clone(), pending.clone()) },
                _ => quote! { layers::Normal::new(state.clone(), pending.clone()) },
            };
            quote! {
                router.method(#rpc_name, S::#handler, #layer);
            }
        })
        .collect();

    quote! {
        mod generated {
            use std::sync::Arc;
            use std::future::{Future, Ready};

            use lsp_types::*;
            use lsp_types::notification::*;
            use lsp_types::request::*;

            use super::#trait_name;
            use crate::jsonrpc::{Result, Router};
            use crate::service::{layers, Client, Pending, ServerState, State, ExitedError};

            fn cancel_request(params: CancelParams, p: &Pending) -> Ready<()> {
                p.cancel(&params.id.into());
                std::future::ready(())
            }

            pub(crate) fn register_lsp_methods<S>(
                mut router: Router<S, ExitedError>,
                state: Arc<ServerState>,
                pending: Arc<Pending>,
                client: Client,
            ) -> Router<S, ExitedError>
            where
                S: #trait_name,
            {
                #route_registrations

                let p = pending.clone();
                router.method(
                    "$/cancelRequest",
                    move |_: &S, params| cancel_request(params, &p),
                    tower::layer::util::Identity::new(),
                );
                router.method(
                    "exit",
                    |_: &S| std::future::ready(()),
                    layers::Exit::new(state.clone(), pending, client.clone()),
                );

                router
            }
        }
    }
}

/// Transforms all `async fn()` to `fn()` returning `impl Future + Send`
fn require_methods_return_send_future(items: &mut Vec<TraitItem>) {
    let empty_type: Box<Type> = syn::parse2(quote!(())).unwrap();

    for method in items {
        let TraitItem::Fn(func) = method else {
            continue;
        };

        let (arrow, rtype) = match func.sig.output.clone() {
            ReturnType::Default => (syn::Token![->](Span::call_site()), empty_type.clone()),
            ReturnType::Type(arrow, rtype) => (arrow, rtype),
        };

        if let Some(blk) = func.default.take() {
            let span = blk.span();
            let stmts = blk.stmts;
            func.default =
                Some(syn::parse2(quote_spanned!(span => { async { #(#stmts)* }})).unwrap());
        }

        let span = rtype.span();

        func.sig.asyncness = None;
        func.sig.output = ReturnType::Type(
            arrow,
            syn::parse2(
                quote_spanned!(span => impl ::core::future::Future<Output = #rtype> + Send),
            )
            .unwrap(),
        );
    }
}
