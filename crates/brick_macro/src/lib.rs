mod classify;
mod ops;
mod utils;

use classify::{impl_classify_attrs, impl_classify_brick, impl_classify_variant};
use ops::{impl_brick_ops, impl_brick_ops_variant, impl_brick_wrap_variant};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, parse_macro_input};

fn into_ts(result: syn::Result<TokenStream2>) -> TokenStream {
    match result {
        Ok(output_stream2) => output_stream2.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(Wrap)]
pub fn brick_wrap(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match ast.data {
        Data::Enum(_) => into_ts(impl_brick_wrap_variant(&ast)),
        _ => syn::Error::new(ast.ident.span(), "Wrap only supports enums")
            .to_compile_error()
            .into(),
    }
}

#[proc_macro_derive(BrickOps, attributes(render_brick))]
pub fn brick_props(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match ast.data {
        Data::Struct(_) => into_ts(impl_brick_ops(&ast)),
        Data::Enum(_) => into_ts(impl_brick_ops_variant(&ast)),
        _ => syn::Error::new(ast.ident.span(), "BrickOps only supports structs and enums")
            .to_compile_error()
            .into(),
    }
}

#[proc_macro_derive(ClassifyAttrs)]
pub fn classify_attrs(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match impl_classify_attrs(&ast) {
        Ok(output_stream2) => output_stream2.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(ClassifyBrick)]
pub fn classify_brick(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match impl_classify_brick(&ast) {
        Ok(output_stream2) => output_stream2.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(ClassifyVariant)]
pub fn classify_variant(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match impl_classify_variant(&ast) {
        Ok(output_stream2) => output_stream2.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn info(_args: TokenStream, input: TokenStream) -> TokenStream {
    // let item = parse_macro_input!(input as Item);
    // quote! {#item}.into()
    input
}

#[cfg(test)]
#[path = "test.rs"]
mod test_macro;
