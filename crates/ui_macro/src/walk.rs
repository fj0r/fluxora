use crate::configlist::ConfigList;
use crate::utils::{get_ident_from_type, struct_has_field};
use std::collections::HashMap;
use std::io::Write;
use syn::Meta;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    pub has_id: bool,
}

#[derive(Debug)]
pub enum CompInfo {
    Struct { name: String, has_sub: bool },
    Enum { fields: Vec<Field> },
}

pub fn walk(ast: &syn::File) -> HashMap<String, CompInfo> {
    #[cfg(test)]
    let mut lf = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("../../data/walklog.rs")
        .unwrap();
    ast.items.iter().fold(HashMap::new(), |mut acc, x| {
        match x {
            syn::Item::Struct(x) => {
                let has_sub = struct_has_field(x, "sub");
                let info = CompInfo::Struct {
                    name: x.ident.to_string(),
                    has_sub,
                };
                acc.insert(x.ident.to_string(), info);
            }
            syn::Item::Enum(x) => {
                let fields = x
                    .variants
                    .iter()
                    .map(|x| {
                        let ty = x
                            .fields
                            .iter()
                            .map(|x| get_ident_from_type(&x.ty))
                            .collect::<Vec<_>>();
                        let ty = if let Some(t) = ty.get(0)
                            && let Some(i) = t
                        {
                            i.to_string()
                        } else {
                            "".to_string()
                        };
                        let kv = x
                            .attrs
                            .iter()
                            .flat_map(|x| {
                                if let Meta::List(x) = &x.meta {
                                    let tk = x.tokens.clone();
                                    #[cfg(test)]
                                    let _ = write!(lf, "{:#?}\n", tk);
                                    syn::parse2::<ConfigList>(tk).unwrap_or_default().0
                                } else {
                                    ConfigList::default().0
                                }
                            })
                            .collect::<HashMap<_, _>>();

                        let has_id = if let Some(h) = kv.get("has_id") {
                            h == "true"
                        } else {
                            false
                        };
                        Field {
                            name: x.ident.to_string(),
                            r#type: ty,
                            has_id,
                        }
                    })
                    .collect();
                acc.insert(x.ident.to_string(), CompInfo::Enum { fields });
            }
            _ => {}
        }
        acc
    })
}
