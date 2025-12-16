use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashMap;
use syn::{Error, parse_file, parse_macro_input};
mod attrs;
use attrs::Attrs;
mod walk;
use walk::{CompInfo, walk};
mod utils;
use std::fs::read_to_string;

macro_rules! bail {
    ($($x: tt)*) => {
        return Error::new(Span::call_site(), format!($($x)*))
            .into_compile_error()
            .into()
    };
}

#[proc_macro]
pub fn gen_dispatch(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as Attrs)
        .0
        .into_iter()
        .collect::<HashMap<_, _>>();

    let Some(file) = config.get("file") else {
        bail!("must provide file");
    };
    println!("cargo:rerun-if-changed={}", file);

    let Some(entry) = config.get("entry") else {
        bail!("must provide entry");
    };

    let Some(object) = config.get("object") else {
        bail!("must provide object");
    };

    let txt = match read_to_string(file) {
        Ok(txt) => txt,
        Err(e) => {
            bail!("{}", e);
        }
    };

    let Ok(ast) = parse_file(&txt) else {
        bail!("parse {} failed", file);
    };

    let Ok(m) = gen_match(&ast, entry, object) else {
        bail!("gen match failed");
    };

    m.into()
}

fn gen_match(ast: &syn::File, entry: &str, object: &str) -> syn::Result<TokenStream2> {
    let info = walk(ast);
    let ty = Ident::new(entry, Span::call_site());
    let ob = Ident::new(object, Span::call_site());
    let CompInfo::Enum { fields } = info
        .get(entry)
        .ok_or(syn::Error::new(Span::call_site(), "no fields"))?
    else {
        return Err(syn::Error::new(Span::call_site(), "no enum"));
    };
    let f = fields.iter().map(|x| {
        let var = Ident::new(&x.name, Span::call_site());
        let var_ = Ident::new(&format!("{}_", &x.name), Span::call_site());
        let has_child = info
            .get(&x.r#type)
            .ok_or(syn::Error::new(Span::call_site(), "get child failed"));
        let Ok(CompInfo::Struct { name: _, has_sub }) = has_child else {
            // return Err(syn::Error::new(Span::call_site(), "no child"));
            panic!("no child")
        };
        let children = if *has_sub {
            quote! {
                {children}
            }
        } else {
            quote! {}
        };
        let id = if x.has_id {
            let id = Ident::new(&format!("{}_id", &x.name).to_uppercase(), Span::call_site());
            let fmt = format!("{}-{{}}", &x.name);
            quote! {
                static #id: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(0));
                let mut tc = #id.lock().unwrap();
                *tc += 1;
                let id = format!(#fmt , *tc) ;
            }
        } else {
            quote! {}
        };

        quote! {
            #ty::#var(c) => {
                #id
                rsx!(#var_ {
                    id: id,
                    brick: c,
                    #children
                })
            }
        }
    });

    Ok(quote! {
        match #ob {
            #(#f),*
        }
    })
}

#[cfg(test)]
#[path = "test.rs"]
mod tests;
