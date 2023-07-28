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

use proc_macro::TokenStream;
use quote::{format_ident, quote, IdentFragment, ToTokens};
use syn::{
    self, parse_macro_input, AttributeArgs, DeriveInput, Ident, Meta, MetaList, MetaNameValue,
    NestedMeta, Type,
};

fn impl_kv_storage(ast: &syn::DeriveInput, kv_struct: MetaList) -> TokenStream {
    // fn impl_kv_storage(ast: &syn::DeriveInput) -> TokenStream {
    let input_struct = match &ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!(),
    };

    let input_struct_vis = &ast.vis;

    let input_struct_name = &ast.ident;

    let (storage, storage_items) = {
        let fields = match &input_struct.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!(),
        };

        let (storage_fields, storage_items): (Vec<_>, Vec<_>) = fields
            .iter()
            .map(|field| {
                let item_name = field.ident.as_ref().unwrap();
                let item_type = field.ty.clone();

                // match field.attrs.iter().next().unwrap().path

                let is_map = field
                    .attrs
                    .iter()
                    .next()
                    .is_some_and(|attr| attr.path.is_ident("map"));

                let is_subpath = field
                    .attrs
                    .iter()
                    .next()
                    .is_some_and(|attr| attr.path.is_ident("subpath"));

                let item_struct = format_ident!("{}{}{}", if is_map { "StorageMap" } else { "StorageItem" },input_struct_name, item_name);

                (
                    // Field for main Storage struct
                    if is_map {
                        quote! {
                            pub fn #item_name(key: &str) -> #item_struct {
                                #item_struct(format!("{}.{}", stringify!(#item_name), key))
                            }
                        }
                    } else if is_subpath {
                        quote! {
                            pub fn #item_name() -> #item_struct {
                                #item_struct(format!("{}.{}", stringify!(#item_name), key))
                            }
                        }
                    } else {
                        quote! {
                            pub fn #item_name() -> #item_struct {
                                #item_struct(String::from(stringify!(#item_name)))
                            }
                        }
                    },
                    if is_map && false {
                        quote! {
                            // #input_struct_vis struct #item_struct(pub &'static str);
                            #input_struct_vis struct #item_struct(pub String);

                            impl #item_struct {
                                pub async fn set_value(&self, key: &str, value: #item_type) {
                                    #kv_struct::put::<#item_type>(&format!("{}.{}", self.0, key), &value).await;
                                }

                                pub async fn value(&self, key: &str) -> #item_type {
                                    #kv_struct::get(&format!("{}.{}", self.0, key)).await
                                }

                                // async fn update<'b, F: FnOnce(&mut #item_type)>(&self, update_fn: F) {
                                //     let mut value = self.value().await;
                                //     update_fn(&mut value);
                                //     self.set_value(value).await;
                                // }
                            }

                            // impl StorageItem<Kv> for #item_struct {
                            //     type Value<'b> = #item_type;
                            //
                            //     fn key(&self) -> &'static str {
                            //         self.0
                            //     }
                            // }
                        }
                    } else {
                        // StorageItem implementation
                        quote! {
                            // #input_struct_vis struct #item_struct(pub &'static str);
                            #input_struct_vis struct #item_struct(pub String);

                            impl #item_struct {
                                pub async fn set_value(&self, value: #item_type) {
                                    #kv_struct::put::<#item_type>(&self.0, &value).await;
                                }

                                pub async fn value(&self) -> #item_type {
                                    #kv_struct::get(&self.0).await
                                }

                                // async fn update<'b, F: FnOnce(&mut #item_type)>(&self, update_fn: F) {
                                //     let mut value = self.value().await;
                                //     update_fn(&mut value);
                                //     self.set_value(value).await;
                                // }
                            }

                            // impl StorageItem<Kv> for #item_struct {
                            //     type Value<'b> = #item_type;
                            //
                            //     fn key(&self) -> &'static str {
                            //         self.0
                            //     }
                            // }
                        }
                    },
                )
            })
            .unzip();

        let storage = quote! {
            #input_struct_vis struct #input_struct_name;

            impl #input_struct_name {
                #(#storage_fields)*
            }
        };

        (storage, storage_items)
    };

    let gen = quote! {
        // use crate::contract_utils::js_imports::Kv;
        // use kv_storage::StorageItem;

        #storage

        #(#storage_items)*

        // fn hey() {
        //     println!("{}", stringify!(#(#fields)*));
        // }
    };

    gen.into()
}

// fn impl_kv_storage(ast: &syn::DeriveInput) -> TokenStream {
//     let input_struct = match &ast.data {
//         syn::Data::Struct(data) => data,
//         _ => panic!(),
//     };
//
//     let input_struct_vis = &ast.vis;
//
//     let input_struct_name = &ast.ident;
//
//     let (storage, storage_items) = {
//         let fields = match &input_struct.fields {
//             syn::Fields::Named(fields) => &fields.named,
//             _ => panic!(),
//         };
//
//         let (storage_fields, storage_items): (Vec<_>, Vec<_>) = fields
//             .iter()
//             .map(|field| {
//                 let item_name = field.ident.as_ref().unwrap();
//                 let item_type = field.ty.clone();
//                 let item_struct = format_ident!("{}{}", "StorageItem", item_name);
//
//                 (
//                     // Field for main Storage struct
//                     quote! {
//                         pub fn #item_name() -> #item_struct {
//                             #item_struct(stringify!(#item_name))
//                         }
//                     },
//                     // StorageItem implementation
//                     quote! {
//                         #input_struct_vis struct #item_struct(pub &'static str);
//
//                         impl StorageItem<Kv> for #item_struct {
//                             type Value<'b> = #item_type;
//
//                             fn key(&self) -> &'static str {
//                                 self.0
//                             }
//                         }
//                     },
//                 )
//             })
//             .unzip();
//
//         let storage = quote! {
//             #input_struct_vis struct #input_struct_name;
//
//             impl #input_struct_name {
//                 #(#storage_fields)*
//             }
//         };
//
//         (storage, storage_items)
//     };
//
//     let gen = quote! {
//         #storage
//
//         #(#storage_items)*
//     };
//
//     gen.into()
// }

// fn impl_kv_storage(ast: &syn::DeriveInput) -> TokenStream {
//     let input_struct = match &ast.data {
//         syn::Data::Struct(data) => data,
//         _ => panic!(),
//     };
//
//     let input_struct_vis = &ast.vis;
//
//     let input_struct_name = &ast.ident;
//
//     let (storage, storage_items) = {
//         let fields = match &input_struct.fields {
//             syn::Fields::Named(fields) => &fields.named,
//             _ => panic!(),
//         };
//
//         let (storage_fields, storage_items): (Vec<_>, Vec<_>) = fields
//             .iter()
//             .map(|field| {
//                 let item_name = field.ident.as_ref().unwrap();
//                 let item_type = field.ty.clone();
//                 let item_struct = format_ident!("{}{}", "StorageItem", item_name);
//
//                 (
//                     // Field for main Storage struct
//                     quote! {
//                         pub fn #item_name() -> #item_struct {
//                             #item_struct(stringify!(#item_name))
//                         }
//                     },
//                     // StorageItem implementation
//                     quote! {
//                         #input_struct_vis struct #item_struct(pub &'static str);
//
//                         impl StorageItem<Kv> for #item_struct {
//                             type Value<'b> = #item_type;
//
//                             fn key(&self) -> &'static str {
//                                 self.0
//                             }
//                         }
//                     },
//                 )
//             })
//             .unzip();
//
//         let storage = quote! {
//             #input_struct_vis struct #input_struct_name;
//
//             impl Storage {
//                 #(#storage_fields)*
//             }
//         };
//
//         (storage, storage_items)
//     };
//
//     let gen = quote! {
//         #storage
//
//         #(#storage_items)*
//
//         // fn hey() {
//         //     println!("{}", stringify!(#(#fields)*));
//         // }
//     };
//
//     gen.into()
// }

#[proc_macro_attribute]
pub fn kv_storage(args: TokenStream, input: TokenStream) -> TokenStream {
    // this was commented
    //
    // // Construct a representation of Rust code as a syntax tree
    // // that we can manipulate
    // let args_ast = parse_macro_input!(args as syn::AttributeArgs);
    // let input_ast = parse_macro_input!(input as DeriveInput);
    //
    // let subkey = {
    //     let attr = args_ast.into_iter().next().unwrap();
    //
    //     match attr {
    //         NestedMeta::Meta(meta) => meta,
    //         _ => panic!("Expected a key-value pair"),
    //     }
    // };
    //
    // // let subkey = match subkey {
    // //     syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name_value)) => match meta_name_value.lit {
    // //         syn::Lit::Str(ref lit_str) => lit_str.value(),
    // //         _ => panic!("Expected a string literal"),
    // //     },
    // //     _ => panic!("Expected a key-value pair"),
    // // };
    //
    // // Build the trait implementation
    // // impl_kv_storage(&input_ast)
    //
    // quote! {
    //     fn hey() {
    //         println!("attr: '{:?}'", #subkey);
    //     }
    //
    //     #input_ast
    // }
    // .into()

    // this was uncommented
    //
    // let args = parse_macro_input!(args as AttributeArgs);
    // let input = parse_macro_input!(input as syn::Item);
    //
    // // let subkey = args.into_iter().next().unwrap();
    // let subkey = match args.get(0) {
    //     Some(syn::NestedMeta::Meta(syn::Meta::Path(path))) => path.is_ident("subkey"),
    //     _ => panic!("Expected a key-value pair"),
    // };
    //
    // // Use the `quote` crate to generate the code based on the `input` and `subkey` values
    // let expanded = quote! {
    //     // Example expansion using the `input` and `subkey` values
    //     fn foo() {
    //         println!("The subkey is: {}", #subkey);
    //         #input
    //     }
    // };
    //
    // proc_macro::TokenStream::from(expanded)

    // try again

    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let params = parse_macro_input!(args as MetaList);
    let input_ast = parse_macro_input!(input as DeriveInput);

    // params.nested.iter().find(|nestedMeta| match nestedMeta {
    //     NestedMeta::Meta(meta) => meta.path,
    //     NestedMeta::Lit(lit) => lit.is,
    // });

    // let subkey = match subkey {
    //     syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name_value)) => match meta_name_value.lit {
    //         syn::Lit::Str(ref lit_str) => lit_str.value(),
    //         _ => panic!("Expected a string literal"),
    //     },
    //     _ => panic!("Expected a key-value pair"),
    // };

    // Build the trait implementation
    impl_kv_storage(&input_ast, params)
    // impl_kv_storage(&input_ast)

    // quote! {
    //     #input_ast
    // }
    // .into()
}
