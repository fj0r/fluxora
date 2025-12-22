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
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

macro_rules! syerr {
    (-> $($x: tt)*) => {
        return Err(syerr!($($x)*))
    };
    ($($x: tt)*) => {
        syn::Error::new(
            Span::call_site(),
            format!($($x)*),
        )
    };
}

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
    let file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(file);

    let Some(entry) = config.get("entry") else {
        bail!("must provide entry");
    };

    let Some(object) = config.get("object") else {
        bail!("must provide object");
    };

    let Ok(m) = gen_match(file, entry, object) else {
        bail!("gen match failed");
    };

    m.into()
}

fn gen_match(file: impl AsRef<Path>, entry: &str, object: &str) -> syn::Result<TokenStream2> {
    let file = file.as_ref().to_str().unwrap();
    let txt = match read_to_string(&file) {
        Ok(txt) => txt,
        Err(e) => {
            syerr!(-> "{}", e);
        }
    };
    let Ok(ast) = parse_file(&txt) else {
        syerr!(-> "parse {:#?} failed", &file);
    };
    let info = walk(&ast);
    let ty = Ident::new(entry, Span::call_site());
    let ob = Ident::new(object, Span::call_site());
    let CompInfo::Enum { fields } = info.get(entry).ok_or(syerr!("no fields"))? else {
        syerr!(-> "no enum");
    };
    let f = fields.iter().map(|x| {
        let var = Ident::new(&x.name, Span::call_site());
        let var_ = Ident::new(&format!("{}_", &x.name), Span::call_site());
        let has_child = info.get(&x.r#type).expect("get child failed");
        let CompInfo::Struct {
            name: _, has_sub, ..
        } = has_child
        else {
            panic!("not a CompInfo::Struct")
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
        const _: &[u8] = include_bytes!(#file);

        match #ob {
            #(#f),*
        }
    })
}

#[cfg(test)]
#[path = "test.rs"]
mod tests;
