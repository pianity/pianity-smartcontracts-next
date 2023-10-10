use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    self,
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, AttributeArgs, DataStruct, DeriveInput, Ident, Meta, NestedMeta,
    Type, Visibility,
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

fn capitalize(string: &str) -> String {
    let (first, rest) = string.split_at(1);
    format!("{}{}", first.to_ascii_uppercase(), rest)
}

fn to_camel_case(string: &str) -> String {
    string
        .split('_')
        .map(capitalize)
        .collect::<Vec<_>>()
        .join("")
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

fn gen_field_peek(
    field_name: &Ident,
    field_args: &FieldArgs,
    macro_args: &MacroArgs,
    return_type: &Ident,
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

    let kv_struct = &macro_args.kv;

    if !field_args.subpath {
        quote! {
            pub async fn #field_name(#(#fun_args),*) -> Option<#return_type> {
                #kv_struct::get::<#return_type>(&format!(#format_str, #(#path_args),*)).await
            }
        }
    } else {
        quote! {
            pub fn #field_name(#(#fun_args),*) -> #return_type {
                #return_type(format!(#format_str, #(#path_args),*))
            }
        }
    }
}

fn create_peek_struct(
    struct_data: &DataStruct,
    struct_visibility: &Visibility,
    struct_ident: &Ident,
    macro_args: &MacroArgs,
) -> TokenStream {
    let struct_fields = match &struct_data.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => panic!("Only named fields are supported"),
    };

    let peek_methods: Vec<_> = struct_fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = if let syn::Type::Path(type_path) = &field.ty {
                type_path.path.segments.last().unwrap().ident.clone()
            } else {
                panic!("I'm afraid I cannot let you do that, Dave");
            };
            let field_args: FieldArgs = field.attrs.clone().into();

            let return_type = if !field_args.subpath {
                field_type
            } else {
                format_ident!("Peek{}", &field_type)
            };

            gen_field_peek(field_name, &field_args, &macro_args, &return_type)
        })
        .collect();

    let peek_struct_name = format_ident!("{}{}", "Peek", struct_ident);

    let peek_struct = quote! {
        #struct_visibility struct #peek_struct_name(pub String);

        impl #peek_struct_name {
            #(#peek_methods)*
        }
    };

    peek_struct
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
                    "{}{}{}",
                    "StorageItem",
                    root_struct_name,
                    to_camel_case(&field_name.to_string())
                );

                let return_type = if field_args.subpath {
                    if let syn::Type::Path(type_path) = &field_type {
                        let ident = type_path.path.segments.last().unwrap().ident.clone();
                        format_ident!("Subpath{}", &ident)
                    } else {
                        panic!("I'm afraid I cannot let you do that, Dave");
                    }
                } else {
                    field_struct_name.clone()
                };

                let field_maybe_struct_name = format_ident!("{}{}", "Maybe", field_struct_name);
                let field_maybe_struct = if field_args.map {
                    let field_type_name = if let syn::Type::Path(type_path) = &field.ty {
                        type_path.path.segments.last().unwrap().ident.clone()
                    } else {
                        panic!("I'm afraid I cannot let you do that, Dave");
                    };
                    let peek_struct_name = format_ident!("{}{}", "Peek", field_type_name);
                    let peek_method = if field_args.subpath {
                        quote! {
                            pub fn peek(&self) -> #peek_struct_name {
                                #peek_struct_name(self.0.clone())
                            }
                        }
                    } else {
                        quote! {
                            pub async fn peek(&self) -> Option<#field_type> {
                                #kv_struct::get::<#field_type>(&self.0).await
                            }
                        }
                    };

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

                    let init_default_steps = if !field_args.subpath {
                        quote! {
                            #kv_struct::put::<#field_type>(&self.0, &#field_type::default()).await;
                        }
                    } else {
                        quote! {
                            #field_type::default().init(self.0.clone()).await;
                            #kv_struct::put::<u8>(&format!("{}.-", self.0), &1).await;
                        }
                    };

                    let map_method = if !field_args.subpath {
                        quote! {
                            pub async fn map<F>(&self, map_fn: F) -> &Self
                            where
                                F: FnOnce(#field_type) -> #field_type,
                            {
                                let value = #kv_struct::get::<#field_type>(&self.0).await;

                                if let Some(value) = value {
                                    #kv_struct::put::<#field_type>(&self.0, &map_fn(value)).await;
                                }

                                self
                            }
                        }
                    } else {
                        quote!()
                    };

                    quote! {
                        struct #field_maybe_struct_name(pub String);

                        impl #field_maybe_struct_name {
                            pub async fn exists(&self) -> bool {
                                #exists_steps
                            }

                            pub async fn init(&self, default: #field_type) -> #return_type {
                                if !self.exists().await {
                                    #init_steps
                                }

                                #return_type(self.0.clone())
                            }

                            pub async fn init_default(&self) -> #return_type {
                                if !self.exists().await {
                                    #init_default_steps
                                }

                                #return_type(self.0.clone())
                            }

                            #peek_method

                            #map_method
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
                    // Implementation of StorageItem or StorageMap
                    if field_args.subpath {
                        quote! {
                            #field_maybe_struct
                        }
                    } else {
                        quote! {
                            #field_maybe_struct

                            #root_struct_vis struct #field_struct_name(pub String);

                            impl #field_struct_name {
                                pub async fn get(&self) -> #field_type {
                                    #kv_struct::get(&self.0).await.unwrap()
                                }

                                pub async fn put(&self, value: &#field_type) {
                                    #kv_struct::put::<#field_type>(&self.0, value).await;
                                }

                                pub async fn map<F>(&self, map_fn: F) -> &Self
                                where
                                    F: FnOnce(#field_type) -> #field_type,
                                {
                                    let value = #kv_struct::get::<#field_type>(&self.0).await;

                                    if let Some(value) = value {
                                        #kv_struct::put::<#field_type>(&self.0, &map_fn(value)).await;
                                    }

                                    self
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
                #[derive(Default, Serialize, Deserialize)]
                #root_struct_vis struct #root_struct_name {
                    #(#cons_fields),*
                }

                impl #root_struct_name {
                    #init_method

                    #(#storage_fields)*
                }
            }
        } else {
            let accessor_struct_name = format_ident!("Subpath{}", root_struct_name);

            quote! {
                #[derive(Default, Serialize, Deserialize)]
                #root_struct_vis struct #root_struct_name {
                    #(#cons_fields),*
                }

                impl #root_struct_name {
                    #init_method
                }

                #root_struct_vis struct #accessor_struct_name(pub String);

                impl #accessor_struct_name {
                    #(#storage_fields)*
                }
            }
        };

        (storage, storage_items)
    };

    let peek_struct =
        create_peek_struct(root_struct, root_struct_vis, root_struct_name, &macro_args);

    let gen = quote! {
        #storage

        #(#storage_items)*

        #peek_struct
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
