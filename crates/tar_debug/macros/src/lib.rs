extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, Stmt, Expr};
use quote::quote;

#[proc_macro_attribute]
pub fn trace(_: TokenStream, stream: TokenStream) -> TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(stream as ItemFn);

    let name = format!("\"{}\"", sig.ident);
    let len = block.stmts.len();

    if len < 1 {
        return quote! {
            #(#attrs)* #vis #sig {
                let __tracer__ = Trace::new(#name);
                __tracer__.end();
            }
        }.into();
    }

    let stmts = &block.stmts[..len-1];
    let last = &block.stmts[len-1];

    let as_last_statement = match last {
        Stmt::Local(_) => false,
        Stmt::Expr(expr) => match expr {
            Expr::Return(_) => false,
            _ => true
        }
        _ => true
    };

    if as_last_statement {
        quote! {
            #(#attrs)* #vis #sig {
                let __tracer__ = Trace::new(#name);
                #(#stmts)*
                __tracer__.end();
                #last
            }
        }
    } else {
        quote! {
            #(#attrs)* #vis #sig {
                let __tracer__ = Trace::new(#name);
                #(#stmts)*
                #last
                __tracer__.end();
            }
        }
    }.into()
}

