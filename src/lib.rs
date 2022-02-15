// implemnt convert struct from xmlrpc::Value::Struct
use std::convert::From;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use quote::ToTokens;
use syn::parse_macro_input;
use syn::DataStruct;
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(Convert)]
pub fn derive_convert(input: TokenStream) -> TokenStream {
    // input token
    let input = parse_macro_input!(input as DeriveInput);
    // struct name
    let struct_name = &input.ident;

    let expanded = match input.data {
        // only name
        Data::Struct(DataStruct { ref fields, .. }) => {
            if let Fields::Named(ref fields) = fields {

                // value to struct confidition
                let confidition_implments =
                    TokenStream2::from_iter(fields.named.iter().map(|field| {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = &field.ty;

                        if field_type.clone().into_token_stream().to_string() == "Vec < String >" {
                            quote! {
                                if k == stringify!(#field_name) {
                                    if let Value::Array(v) = v {
                                        for v in v.into_iter() {
                                            if let Value::String(v) = v {
                                                result.#field_name.push(v.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        } else if field_type.clone().into_token_stream().to_string() == "i32" {
                            quote! {
                                if k == stringify!(#field_name) {
                                    if let Value::Int(v) = v {
                                        result.#field_name = v;
                                    }
                                }
                            }
                        }else {
                            quote! {
                            if k == stringify!(#field_name) {
                                if let Value::#field_type(v) = v {
                                    result.#field_name = v.clone();
                                    continue;
                                }
                            }
                            }
                        }
                    }));
                // struct to value confidition
                let s2v_confidition = TokenStream2::from_iter(fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;

                    // need to process extra type for 'Vec<String>'
                    if field_type.clone().into_token_stream().to_string() == "Vec < String >" {
                        quote! {
                            let mut v = Vec::<Value>::new();
                            for cate in post.#field_name.into_iter() {
                                v.push(Value::String(cate));
                            }
                            hashmap.insert(stringify!(#field_name).to_string(), Value::Array(v));
                        }
                    } else if field_type.clone().into_token_stream().to_string() == "i32"{
                        quote! {
                            hashmap.insert(stringify!(#field_name).to_string(), Value::Int(post.#field_name));
                        }

                    } else {
                        quote! {
                            hashmap.insert(stringify!(#field_name).to_string(), Value::#field_type(post.#field_name));
                        }
                    }
                }));
                let implemented_convert = quote! {
                    // implement convert Value to Struct
                    impl From<Value> for #struct_name {
                        fn from(value: Value) -> Self {
                            let mut result = Self::default();
                            match value {
                                Value::Struct(value) => {
                                    for (k, v) in value.into_iter() {
                                        #confidition_implments
                                    }
                                },
                            _ => panic!("result must be struct"),
                            }
                            result
                        }
                    }

                    // implement convert Struct to Value
                    impl From<#struct_name> for Value {
                        fn from(post: #struct_name) -> Self {
                            let mut hashmap = BTreeMap::new();
                            #s2v_confidition
                            Value::Struct(hashmap)
                        }
                    }
                };
                implemented_convert
            } else {
                panic!("sorry, may it's a complicated struct.");
            }
        }
        _ => panic!("Convert is not implemented for union or enum type!"),
    };
    expanded.into()
}
