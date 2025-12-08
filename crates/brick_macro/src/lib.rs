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
mod test_macro {
    use super::*;
    use quote::quote;
    use syn::DeriveInput;
    use syn::parse2;

    #[test]
    fn test_struct_hello() {
        let input = quote! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
            #[cfg_attr(feature = "dioxus", derive(Props))]
            #[cfg_attr(feature = "schema", derive(JsonSchema))]
            pub struct Placeholder {
                #[serde(skip_serializing_if = "Option::is_none")]
                pub id: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub attrs: Option<ClassAttr>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub children: Option<Vec<Brick>>,
            }
        };

        let _input = quote! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
            #[cfg_attr(feature = "dioxus", derive(Props))]
            #[cfg_attr(feature = "schema", derive(JsonSchema))]
            pub struct ClassAttr {
                #[serde(skip_serializing_if = "Option::is_none")]
                pub class: Option<Vec<String>>,
            }
        };

        let _input = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            #[cfg_attr(feature = "schema", derive(JsonSchema))]
            #[serde(tag = "type")]
            pub enum Brick {
                case(Case),
                #[render_brick(has_id = true)]
                placeholder(Placeholder),
            }
        };

        let _input = quote! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            #[cfg_attr(feature = "schema", derive(JsonSchema))]
            #[serde(untagged)]
            pub enum BindVariant {
                Source {
                    source: String,
                },
                Target {
                    target: String,
                },
                Event {
                    event: String,
                },
                Field {
                    field: String,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    payload: Option<Value>,
                    #[cfg(feature = "dioxus")]
                    #[allow(dead_code)]
                    #[serde(skip)]
                    signal: Option<Signal<Value>>,
                },
                Submit {
                    submit: bool,
                    #[cfg(feature = "dioxus")]
                    #[allow(dead_code)]
                    #[serde(skip)]
                    signal: Option<Signal<Value>>,
                },
                Default {},
            }
        };

        //let output = impl_classify_attrs(input.clone()).unwrap();
        let ast = syn::parse2::<DeriveInput>(input).unwrap();
        let output = impl_brick_ops(&ast).expect("Macro expansion failed");

        let _ = std::fs::write("../data/out.ast", format!("{:#?}", ast));
        let _ = std::fs::write("../data/out.rs", format!("{:#}", output.to_string()));

        assert!(true);
    }

    #[test]
    fn test_attribute_rename() {
        use syn::ItemFn;
        let input_args = quote! { value=1 };

        let input_item = quote! {
            #[xxx]
            fn original_function() {
                println!("{:?}", 123);
            }
        };

        let expected_fn: ItemFn = parse2(input_item).expect("Failed to parse expected output");

        let _ = std::fs::write("../data/itemfn.ast", format!("{:#?}", expected_fn));
    }
}
