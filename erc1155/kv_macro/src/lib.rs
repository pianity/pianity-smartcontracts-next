// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, AttributeArgs, ItemStruct, Meta, NestedMeta};
//
// #[proc_macro_attribute]
// pub fn kv_storage(args: TokenStream, input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as ItemStruct);
//     let args = parse_macro_input!(args as AttributeArgs);
//
//     let mut required_arg_val = None;
//     let mut subpath_present = false; // Now a boolean flag
//
//     for arg in args {
//         match arg {
//             NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("requiredArg") => {
//                 if let syn::Lit::Str(lit) = nv.lit {
//                     required_arg_val = Some(lit.value());
//                 }
//             }
//             NestedMeta::Meta(Meta::Path(p)) if p.is_ident("subpath") => {
//                 subpath_present = true; // simply set to true when present
//             }
//             _ => {}
//         }
//     }
//
//     let expanded = quote! {
//         #input
//         fn logg() {
//             println!("{}, {:?}", #required_arg_val, #subpath_present);
//         }
//     };
//
//     TokenStream::from(expanded)
// }

use std::alloc::GlobalAlloc;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, IdentFragment, ToTokens};
use syn::{
    self, parse_macro_input, Attribute, AttributeArgs, DeriveInput, Expr, Ident, Meta, MetaList,
    MetaNameValue, NestedMeta, Type,
};

struct MacroArgs {
    kv: Ident,
    subpath: bool,
}

impl From<Vec<NestedMeta>> for MacroArgs {
    fn from(nested_metas: Vec<NestedMeta>) -> Self {
        let (subpath, kv) = {
            let mut subpath = false;
            let mut kv = None;

            for arg in nested_metas {
                match arg {
                    NestedMeta::Meta(Meta::Path(p)) if p.is_ident("subpath") => {
                        subpath = true;
                    }
                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("kv") => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            kv = Some(Ident::new(&lit.value(), Span::call_site()));
                        }
                    }
                    _ => panic!("Invalid macro attribute"),
                }
            }

            (subpath, kv)
        };

        Self {
            subpath,
            kv: kv.expect("Required `kv` attribute not provided"),
        }
    }
}

struct FieldArgs {
    map: bool,
    subpath: bool,
}

impl From<Vec<Attribute>> for FieldArgs {
    fn from(attrs: Vec<Attribute>) -> Self {
        let nested_metas = attrs
            .iter()
            .filter_map(|attr| {
                attr.path.is_ident("kv_storage_macro").then(|| {
                    attr.parse_args::<NestedMeta>()
                        .expect("couldn't parse attributes")
                })
            })
            .collect::<Vec<_>>();

        let (map, subpath) = {
            let mut map = false;
            let mut subpath = false;

            for arg in nested_metas {
                match arg {
                    NestedMeta::Meta(Meta::Path(path)) if path.is_ident("map") => {
                        map = true;
                    }
                    NestedMeta::Meta(Meta::Path(path)) if path.is_ident("subpath") => {
                        subpath = true;
                    }
                    _ => panic!("Invalid field attribute"),
                }
            }

            (map, subpath)
        };

        Self { map, subpath }
    }
}

// if field_args.subpath {
//     if macro_args.subpath {
//         quote! {
//             pub fn #field_name(&self) -> #field_type {
//                 #field_type(format!("{}.{}", &self.0, stringify!(#field_name)))
//             }
//         }
//     } else {
//         quote! {
//             pub fn #field_name() -> #field_type {
//                 #field_type(format!(".{}", stringify!(#field_name)))
//             }
//         }
//     }
// } else {
//     if macro_args.subpath {
//         quote! {
//             pub fn #field_name(&self) -> #field_struct_name {
//                 #field_struct_name(format!("{}.{}", &self.0, stringify!(#field_name)))
//             }
//         }
//     } else {
//         quote! {
//             pub fn #field_name() -> #field_struct_name {
//                 #field_struct_name(format!(".{}", stringify!(#field_name)))
//             }
//         }
//     }
// },

fn gen_field_name(
    field: &syn::Field,
    field_args: &FieldArgs,
    macro_args: &MacroArgs,
    field_struct_name: &Ident,
) -> TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_type = &field.ty;

    let mut fun_args: Vec<TokenStream> = Vec::new();
    let mut path_args = Vec::new();

    if macro_args.subpath {
        fun_args.push(quote!(&self));
        path_args.push(quote!(&self.0));
    }

    if field_args.map {
        // fun_args.push(quote!(key: &str));
        // path_args.push(quote!(&key));
    }

    if field_args.subpath {
        // fun_args.push(quote!());
    }

    path_args.push(quote!(stringify!(#field_name)));

    let return_type = if field_args.subpath {
        quote!(#field_type)
    } else {
        quote!(#field_struct_name)
    };

    // if path_args.len() > 1 {
    //     path_args = vec![quote!(format!(#(#path_args),*))];
    // }

    let format_str = match path_args.len() {
        1 => ".{}",
        2 => "{}.{}",
        3 => "{}.{}.{}",
        _ => panic!("Having more than 3 path args is not supported"),
    };

    quote! {
        pub fn #field_name(#(#fun_args),*) -> #return_type {
            #return_type(format!(#format_str, #(#path_args),*))
        }
    }
}

fn impl_kv_storage(ast: &syn::DeriveInput, macro_args: MacroArgs) -> TokenStream {
    // fn impl_kv_storage(ast: &syn::DeriveInput) -> TokenStream {
    let root_struct = match &ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!(),
    };

    let root_struct_vis = &ast.vis;

    let root_struct_name = &ast.ident;

    let (storage, storage_items) = {
        let fields = match &root_struct.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        };

        let (storage_fields, storage_items): (Vec<_>, Vec<_>) = fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = field.ty.clone();

                // match field.attrs.iter().next().unwrap().path

                // let is_map = field
                //     .attrs
                //     .iter()
                //     .next()
                //     .is_some_and(|attr| attr.path.is_ident("map"));

                // let is_subpath = field
                //     .attrs
                //     .iter()
                //     .next()
                //     .unwrap()
                //     .parse_args::<NestedMeta>()
                //     .unwrap();


                let field_args: FieldArgs = field
                    .attrs
                    .clone()
                    .into();

                let kv_struct = &macro_args.kv;

                let field_struct_name = format_ident!(
                    "{}{}{}",
                    if field_args.map {
                        "StorageMap"
                    } else {
                        "StorageItem"
                    },
                    root_struct_name,
                    field_name
                );

                // let field_path = match (macro_args.subpath, field_args.subpath, field_args.map) {
                //     (true, false, false) => quote! {
                //         quote! {
                //             pub fn #field_name(&self) -> #field_type {
                //                 #field_type(format!("{}.{}", &self.0, stringify!(#field_name)))
                //             }
                //         }
                //     },
                //     (true, true, false) => quote! {
                //         quote! {
                //             pub fn #field_name(&self) -> #field_struct_name {
                //                 #field_type(format!("{}.{}", &self.0, stringify!(#field_name)))
                //             }
                //         }
                //     },
                //     (false, false, false) => quote! {
                //         format!(".{}", stringify!(#field_name))
                //     },
                // };

                (
                    // Field for main Storage struct
                    gen_field_name(field, &field_args, &macro_args, &field_struct_name),
                    // if field_args.subpath {
                    //     if macro_args.subpath {
                    //         quote! {
                    //             pub fn #field_name(&self) -> #field_type {
                    //                 #field_type(format!("{}.{}", &self.0, stringify!(#field_name)))
                    //             }
                    //         }
                    //     } else {
                    //         quote! {
                    //             pub fn #field_name() -> #field_type {
                    //                 #field_type(format!(".{}", stringify!(#field_name)))
                    //             }
                    //         }
                    //     }
                    // } else {
                    //     if macro_args.subpath {
                    //         quote! {
                    //             pub fn #field_name(&self) -> #field_struct_name {
                    //                 #field_struct_name(format!("{}.{}", &self.0, stringify!(#field_name)))
                    //             }
                    //         }
                    //     } else {
                    //         quote! {
                    //             pub fn #field_name() -> #field_struct_name {
                    //                 #field_struct_name(format!(".{}", stringify!(#field_name)))
                    //             }
                    //         }
                    //     }
                    // },
                    // Implementation of StorageItem or StorageMap
                    if field_args.map {
                        quote! {
                            #root_struct_vis struct #field_struct_name(pub String);

                            impl #field_struct_name {
                                pub async fn set_value(&self, key: &str, value: #field_type) {
                                    #kv_struct::put::<#field_type>(&format!("{}.{}", self.0, key), &value).await;
                                }

                                pub async fn value(&self, key: &str) -> #field_type {
                                    #kv_struct::get(&format!("{}.{}", self.0, key)).await
                                }

                                // async fn update<'b, F: FnOnce(&mut #item_type)>(&self, update_fn: F) {
                                //     let mut value = self.value().await;
                                //     update_fn(&mut value);
                                //     self.set_value(value).await;
                                // }
                            }
                        }
                    } else if field_args.subpath {
                        quote! { }
                    } else {
                        quote! {
                            #root_struct_vis struct #field_struct_name(pub String);

                            impl #field_struct_name {
                                pub async fn set_value(&self, value: #field_type) {
                                    #kv_struct::put::<#field_type>(&self.0, &value).await;
                                }

                                pub async fn value(&self) -> #field_type {
                                    #kv_struct::get(&self.0).await
                                }

                                // async fn update<'b, F: FnOnce(&mut #item_type)>(&self, update_fn: F) {
                                //     let mut value = self.value().await;
                                //     update_fn(&mut value);
                                //     self.set_value(value).await;
                                // }
                            }
                        }
                    },
                )
            })
            .unzip();

        let storage = if macro_args.subpath {
            quote! {
                #root_struct_vis struct #root_struct_name(pub String);

                impl #root_struct_name {
                    #(#storage_fields)*
                }
            }
        } else {
            quote! {
                #root_struct_vis struct #root_struct_name;

                impl #root_struct_name {
                    #(#storage_fields)*
                }
            }
        };

        (storage, storage_items)
    };

    let gen = quote! {
        // use crate::contract_utils::js_imports::Kv;
        // use kv_storage::StorageItem;

        #storage

        #(#storage_items)*
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn kv_storage(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let args: MacroArgs = parse_macro_input!(args as AttributeArgs).into();
    let input_ast = parse_macro_input!(input as DeriveInput);

    // for arg in args {
    //     match arg {
    //         NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("map") => {
    //             if let syn::Lit::Str(lit) = nv.lit {
    //                 required_arg_val = Some(lit.value());
    //             }
    //         }
    //         NestedMeta::Meta(Meta::Path(p)) if p.is_ident("subpath") => {
    //             subpath_present = true; // simply set to true when present
    //         }
    //         _ => {}
    //     }
    // }

    // Build the trait implementation
    proc_macro::TokenStream::from(impl_kv_storage(&input_ast, args))
    // impl_kv_storage(&input_ast)
}
