use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{parse::Parser, Attribute, Error, Field, Fields, ItemStruct, Result};

use crate::{DEFAULT_CREATE_TIME_NAME, DEFAULT_ID_NAME, DEFAULT_UPDATE_TIME_NAME};

// 公用字段数组：[("sqlheper的属性名称","字段名字")]
const COMMON_FIELDS: [(&str, &str); 3] = [
    (DEFAULT_ID_NAME, DEFAULT_ID_NAME),
    (DEFAULT_CREATE_TIME_NAME, DEFAULT_CREATE_TIME_NAME),
    (DEFAULT_UPDATE_TIME_NAME, DEFAULT_UPDATE_TIME_NAME),
];

pub fn impl_common_fields(ast: &mut ItemStruct) -> TokenStream {
    if let Err(e) = check_field_already_exists(&ast.fields) {
        return e.into_compile_error().into();
    }

    let derive_vec = vec!["Object", "SqlHelper"];

    for derive_name in derive_vec {
        if !check_derive(&ast.attrs, derive_name) {
            return Error::new(
                ast.ident.span(),
                format!("add {} derive [common_fields]", derive_name),
            )
            .into_compile_error()
            .into();
        }
    }

    let id_ident = format_ident!("{}", DEFAULT_ID_NAME);
    let create_time_ident = format_ident!("{}", DEFAULT_CREATE_TIME_NAME);
    let update_time_ident = format_ident!("{}", DEFAULT_UPDATE_TIME_NAME);

    match &mut ast.fields {
        Fields::Named(fields) => {
            let add_fields = vec![
                quote!(
                      #[id]
                      #[oai(read_only)]
                      pub #id_ident:i32
                ),
                quote!(
                      #[create_time]
                      #[oai(read_only)]
                      pub #create_time_ident:NaiveDateTime
                ),
                quote!(
                      #[update_time]
                      #[oai(read_only)]
                      pub #update_time_ident:NaiveDateTime
                ),
            ];
            for add_field in add_fields {
                fields
                    .named
                    .push(Field::parse_named.parse2(add_field).unwrap());
            }
        }
        _ => {
            return Error::new(ast.ident.span(), "only supports struct [common_fields]")
                .into_compile_error()
                .into();
        }
    }

    quote!(#ast).into()
}

fn check_field_already_exists(fields: &Fields) -> Result<()> {
    for field in fields.iter() {
        if let Some(ident) = &field.ident {
            for c_field in COMMON_FIELDS {
                if ident == c_field.1 {
                    return Err(Error::new(
                        ident.span(),
                        format!("`{}` already exists [common_fields]", ident),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn check_derive(attrs: &Vec<Attribute>, derive_name: &str) -> bool {
    //eprintln!("{:?}", attrs);
    for attr in attrs {
        //eprintln!("{:?}", attr);
        if attr.path.is_ident("derive") {
            for seg in attr.tokens.clone() {
                if let TokenTree::Group(group) = seg {
                    for tt in group.stream() {
                        if let TokenTree::Ident(ident) = tt {
                            if ident == derive_name {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}
