use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    self,
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, AttributeArgs, DeriveInput, Ident, Meta, NestedMeta,
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

impl Parse for FieldArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut map = false;
        let mut subpath = false;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
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
            if attr.path.is_ident("kv_storage_macro") {
                let parsed_args = attr.parse_args().expect("shit in the pant");
                output.push(parsed_args);
            }
        }
        Self {
            map: output.iter().any(|args| args.map),
            subpath: output.iter().any(|args| args.subpath),
        }
    }
}

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

    path_args.push(quote!(stringify!(#field_name)));

    if field_args.map {
        fun_args.push(quote!(key: &str));
        path_args.push(quote!(&key));
    }

    let return_type = if field_args.subpath {
        quote!(#field_type)
    } else {
        quote!(#field_struct_name)
    };

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
                    "{}{}{}",
                    if field_args.map {
                        "StorageMap"
                    } else {
                        "StorageItem"
                    },
                    root_struct_name,
                    field_name
                );

                (
                    // Field for main Storage struct
                    gen_field_name(field, &field_args, &macro_args, &field_struct_name),
                    // Implementation of StorageItem or StorageMap
                    if field_args.subpath {
                        quote! {}
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
