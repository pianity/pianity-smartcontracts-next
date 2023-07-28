

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
