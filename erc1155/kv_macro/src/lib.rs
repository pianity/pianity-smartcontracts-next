use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    self,
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, AttributeArgs, DeriveInput, Ident, Meta, NestedMeta, Type,
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
                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("impl") => {
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
            kv: kv.expect("Required `impl` attribute not provided"),
        }
    }
}

struct FieldArgs {
    map: bool,
    subpath: bool,
}

impl Parse for FieldArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut map = false;
        let mut subpath = false;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "map" => map = true,
                "subpath" => subpath = true,
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "Expected `map` or `subpath`",
                    ))
                }
            }

            if !input.is_empty() {
                let _: syn::token::Comma = input.parse()?;
            }
        }

        Ok(Self { map, subpath })
    }
}

impl From<Vec<Attribute>> for FieldArgs {
    fn from(attrs: Vec<Attribute>) -> Self {
        let mut output: Vec<FieldArgs> = Vec::new();
        for attr in attrs {
            if attr.path.is_ident("kv") {
                let parsed_args = attr.parse_args().unwrap();
                output.push(parsed_args);
            } else {
                if let Some(ident) = attr.path.get_ident() {
                    panic!("Invalid attribute: {}", ident.to_string());
                } else {
                    panic!("Invalid attribute");
                }
            }
        }
        Self {
            map: output.iter().any(|args| args.map),
            subpath: output.iter().any(|args| args.subpath),
        }
    }
}

fn gen_field_name(
    field_name: &Ident,
    field_args: &FieldArgs,
    macro_args: &MacroArgs,
    return_type: Ident,
) -> TokenStream {
    let mut fun_args: Vec<TokenStream> = Vec::new();
    let mut path_args = Vec::new();

    if macro_args.subpath {
        fun_args.push(quote!(&self));
        path_args.push(quote!(&self.0));
    }

    path_args.push(quote!(stringify!(#field_name)));

    if field_args.map {
        fun_args.push(quote!(key: &str));
        path_args.push(quote!(&key));
    }

    let format_str = {
        let format_str = match path_args.len() {
            1 => "{}",
            2 => "{}.{}",
            3 => "{}.{}.{}",
            _ => panic!("Having more than 3 path args is not supported"),
        };

        if macro_args.subpath {
            format_str.to_string()
        } else {
            format!(".{}", format_str)
        }
    };

    quote! {
        pub fn #field_name(#(#fun_args),*) -> #return_type {
            #return_type(format!(#format_str, #(#path_args),*))
        }
    }
}

fn impl_kv_storage(ast: &syn::DeriveInput, macro_args: MacroArgs) -> TokenStream {
    let root_struct = match &ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!("Only structs are supported"),
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

                let field_args: FieldArgs = field.attrs.clone().into();

                let kv_struct = &macro_args.kv;

                let field_struct_name = format_ident!(
                    "{}_{}_{}",
                    "StorageItem",
                    root_struct_name,
                    field_name
                );

                let return_type = if field_args.subpath {
                    if let syn::Type::Path(type_path) = &field_type {
                        let ident = type_path.path.segments.last().unwrap().ident.clone();
                        format_ident!("Subpath_{}", &ident)
                    } else {
                        panic!("I'm afraid I cannot let you do that, Dave");
                    }
                } else {
                    field_struct_name.clone()
                };

                let field_maybe_struct_name = format_ident!("{}_{}", "Maybe", field_struct_name);
                let field_maybe_struct = if field_args.map {
                    let exists_steps = if !field_args.subpath {
                        quote! {
                            #kv_struct::get::<u8>(&self.0).await.is_some()
                        }
                    } else {
                        quote! {
                            #kv_struct::get::<u8>(&format!("{}.-", self.0)).await.is_some_and(|v| v == 1)
                        }
                    };

                    let init_steps = if !field_args.subpath {
                        quote! {
                            #kv_struct::put::<#field_type>(&self.0, &default).await;
                        }
                    } else {
                        quote! {
                            default.init(self.0.clone()).await;
                            #kv_struct::put::<u8>(&format!("{}.-", self.0), &1).await;
                        }
                    };

                    quote! {
                        #[allow(non_camel_case_types)]
                        struct #field_maybe_struct_name(pub String);

                        impl #field_maybe_struct_name {
                            pub async fn exists(&self) -> bool {
                                #exists_steps
                            }

                            // pub async fn init_default(&self) -> #field_struct_name {
                            //     if !self.exists().await {
                            //         // #kv_struct::put::<#field_type>(&self.0, &#field_type::default()).await;
                            //     }
                            //
                            //     #field_struct_name(self.0.clone())
                            // }

                            pub async fn init(&self, default: #field_type) -> #return_type {
                                if !self.exists().await {
                                    #init_steps
                                }

                                #return_type(self.0.clone())
                            }

                            // pub async fn else_init<F>(&self, default_fn: F) -> #return_type
                            // where
                            //     F: FnOnce() -> #field_type,
                            // {
                            //     if !self.exists().await {
                            //         #kv_struct::put::<#field_type>(&self.0, &default_fn()).await;
                            //     }
                            //
                            //     #field_struct_name(self.0.clone())
                            // }

                            // pub async fn set_value(&self, value: &#field_type) {
                            //     #kv_struct::put::<#field_type>(&self.0, value).await;
                            // }

                            // pub async fn value(&self) -> Option<#field_type> {
                            //     #kv_struct::get(&self.0).await
                            // }
                        }
                    }
                } else {
                    quote!()
                };

                let return_type = if !field_args.map {
                    return_type
                } else {
                    field_maybe_struct_name
                };

                (
                    // Field for main Storage struct
                    gen_field_name(field_name, &field_args, &macro_args, return_type),
                    // // Implementation of StorageItem or StorageMap
                    if field_args.subpath {
                        quote! {
                            #field_maybe_struct
                        }
                    } else {
                        quote! {
                            #field_maybe_struct

                            #[allow(non_camel_case_types)]
                            #root_struct_vis struct #field_struct_name(pub String);

                            impl #field_struct_name {
                                pub async fn set_value(&self, value: &#field_type) {
                                    #kv_struct::put::<#field_type>(&self.0, value).await;
                                }

                                pub async fn value(&self) -> #field_type {
                                    #kv_struct::get(&self.0).await.unwrap()
                                }
                            }
                        }
                    },
                )
            })
            .unzip();

        /// Transform field type to `HashMap<String, T>` if `map` attribute is set
        fn transform_field_type(field_type: &Type, is_map: bool) -> TokenStream {
            if is_map {
                quote! {
                    std::collections::HashMap<String, #field_type>
                }
            } else {
                quote! {
                    #field_type
                }
            }
        }

        // Construct the fields of the constructor struct
        let cons_fields: Vec<_> = fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_args: FieldArgs = field.attrs.clone().into();
                let field_type = transform_field_type(&field.ty, field_args.map);

                quote! {
                    #field_name: #field_type
                }
            })
            .collect();

        // Construct the init method of the constructor struct, which initializes the KV store
        // fields
        let init_method = {
            let steps = fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_args: FieldArgs = field.attrs.clone().into();
                let field_type = &field.ty;

                let kv_struct = &macro_args.kv;

                if !field_args.map {
                    let path = if !macro_args.subpath {
                        let path_literal = format!(".{}", field_name);
                        quote!(#path_literal)
                    } else {
                        let fmt_literal = format!("{{}}.{}", field_name);
                        quote!(&format!(#fmt_literal, path))
                    };

                    if !field_args.subpath {
                        quote! {
                            #kv_struct::put::<#field_type>(#path, &self.#field_name).await
                        }
                    } else {
                        quote! {
                            self.#field_name.init(String::from(#path)).await
                        }
                    }
                } else {
                    let path = if !macro_args.subpath {
                        let fmt_literal = format!(".{}.{{}}", field_name);
                        quote!(format!(#fmt_literal, key))
                    } else {
                        let fmt_literal = format!("{{}}.{}.{{}}", field_name);
                        quote!(format!(#fmt_literal, path, key))
                    };

                    if !field_args.subpath {
                        quote! {
                            for (key, value) in self.#field_name.iter() {
                                #kv_struct::put::<#field_type>(&#path, &value).await
                            }
                        }
                    } else {
                        quote! {
                            for (key, value) in self.#field_name.iter() {
                                value.init(#path).await;
                                #kv_struct::put::<u8>(&format!("{}.-", #path), &1).await;
                            }
                        }
                    }
                }
            });

            let init_path_arg = if macro_args.subpath {
                quote!(, path: String)
            } else {
                quote!()
            };

            quote! {
                pub async fn init(&self #init_path_arg) {
                    #(#steps;)*
                }
            }
        };

        let storage = if !macro_args.subpath {
            quote! {
                #root_struct_vis struct #root_struct_name {
                    #(#cons_fields),*
                }

                impl #root_struct_name {
                    #init_method

                    #(#storage_fields)*
                }
            }
        } else {
            let accessor_struct_name = format_ident!("Subpath_{}", root_struct_name);

            quote! {
                #root_struct_vis struct #root_struct_name {
                    #(#cons_fields),*
                }

                impl #root_struct_name {
                    #init_method
                }

                #[allow(non_camel_case_types)]
                #root_struct_vis struct #accessor_struct_name(pub String);

                impl #accessor_struct_name {
                    #(#storage_fields)*
                }
            }
        };

        (storage, storage_items)
    };

    let gen = quote! {
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

    proc_macro::TokenStream::from(impl_kv_storage(&input_ast, args))
}
