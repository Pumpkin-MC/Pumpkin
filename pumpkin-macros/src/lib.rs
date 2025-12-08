use proc_macro::TokenStream;
use proc_macro_error2::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{self, Attribute, DeriveInput, LitStr, Type, parse_quote};
use syn::{Block, Expr, Field, Fields, ItemStruct, Stmt, parse_macro_input};

#[proc_macro_derive(Event)]
pub fn event(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics crate::plugin::Payload for #name #ty_generics #where_clause {
            fn get_name_static() -> &'static str {
                stringify!(#name)
            }

            fn get_name(&self) -> &'static str {
                stringify!(#name)
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn cancellable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let name = &item_struct.ident;
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    match &mut item_struct.fields {
        Fields::Named(fields) => {
            if fields
                .named
                .iter()
                .any(|f| f.ident.as_ref().map(|i| i == "cancelled").unwrap_or(false))
            {
                abort!(fields.span(), "Struct already has a `cancelled` field");
            }

            let field: Field = parse_quote! {
                pub cancelled: bool
            };
            fields.named.push(field);
        }
        _ => abort!(
            item_struct.span(),
            "#[cancellable] can only be used on structs with named fields"
        ),
    }

    quote! {
        #item_struct

        impl #impl_generics crate::plugin::Cancellable for #name #ty_generics #where_clause {
            fn cancelled(&self) -> bool {
                self.cancelled
            }

            fn set_cancelled(&mut self, cancelled: bool) {
                self.cancelled = cancelled;
            }
        }
    }
    .into()
}

#[proc_macro_error]
#[proc_macro]
pub fn send_cancellable(input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as Block);

    let mut event_expr = None;
    let mut after_block = None;
    let mut cancelled_block = None;

    for stmt in block.stmts {
        match stmt {
            Stmt::Expr(expr, _) => {
                // Check if it is a labeled block first
                let mut is_special_block = false;
                if let Expr::Block(ref b) = expr
                    && let Some(ref label) = b.label
                {
                    let label_name = label.name.ident.to_string();
                    if label_name == "after" {
                        after_block = Some(b.clone()); // Clone strictly necessary here as we split AST
                        is_special_block = true;
                    } else if label_name == "cancelled" {
                        cancelled_block = Some(b.clone());
                        is_special_block = true;
                    }
                }

                // If it wasn't a special block, it must be the event expression
                if !is_special_block {
                    if event_expr.is_some() {
                        abort!(
                            expr.span(),
                            "Multiple event expressions found. Only one event expression allowed."
                        );
                    }
                    event_expr = Some(expr);
                }
            }
            // Abort on other statements (like `let x = ...`) if strictness is desired
            _ => abort!(
                stmt.span(),
                "Only event expressions and labeled blocks allowed in `send_cancellable!`"
            ),
        }
    }

    let event = match event_expr {
        Some(e) => e,
        None => abort_call_site!("Event expression must be specified"),
    };

    // Construct the if/else logic
    let logic = match (after_block, cancelled_block) {
        (Some(after), Some(cancelled)) => quote! {
            if !event.cancelled { #after } else { #cancelled }
        },
        (Some(after), None) => quote! {
            if !event.cancelled { #after }
        },
        (None, Some(cancelled)) => quote! {
            if event.cancelled { #cancelled }
        },
        (None, None) => quote! {},
    };

    quote! {
        let event = crate::PLUGIN_MANAGER.fire(#event).await;
        #logic
    }
    .into()
}

#[proc_macro_attribute]
pub fn packet(args: TokenStream, item: TokenStream) -> TokenStream {
    let packet_id_expr = parse_macro_input!(args as Expr);
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        #ast
        impl #impl_generics crate::packet::Packet for #name #ty_generics #where_clause {
            const PACKET_ID: i32 = #packet_id_expr;
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn pumpkin_block(args: TokenStream, item: TokenStream) -> TokenStream {
    let arg_lit = parse_macro_input!(args as LitStr);
    let arg_value = arg_lit.value();

    let (namespace, id) = match arg_value.split_once(':') {
        Some(pair) => pair,
        None => {
            return syn::Error::new(
                arg_lit.span(),
                "Expected format \"namespace:id\" (e.g. \"minecraft:stone\")",
            )
            .to_compile_error()
            .into();
        }
    };

    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let code = quote! {
        #ast
        impl #impl_generics crate::block::BlockMetadata for #name #ty_generics #where_clause {
            fn namespace(&self) -> &'static str {
                #namespace
            }
            fn ids(&self) -> &'static [&'static str] {
                &[#id]
            }
        }
    };

    code.into()
}

#[proc_macro_attribute]
pub fn pumpkin_block_from_tag(args: TokenStream, item: TokenStream) -> TokenStream {
    let arg_lit = parse_macro_input!(args as LitStr);
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let full_tag = arg_lit.value();

    // Efficient splitting
    let namespace = match full_tag.split_once(':') {
        Some((ns, _)) => ns,
        None => abort!(arg_lit.span(), "Expected format 'namespace:path'"),
    };

    quote! {
        #ast
        impl #impl_generics crate::block::BlockMetadata for #name #ty_generics #where_clause {
            fn namespace(&self) -> &'static str {
                #namespace
            }
            fn ids(&self) -> &'static [&'static str] {
                get_tag_values(RegistryKey::Block, #arg_lit).unwrap()
            }
        }
    }
    .into()
}

// #[proc_macro_error]
// #[proc_macro_attribute]
// pub fn block_property(input: TokenStream, item: TokenStream) -> TokenStream {
//     let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
//     let name = &ast.ident;
//     let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

//     let input_string = input.to_string();
//     let input_parts: Vec<&str> = input_string.split("[").collect();
//     let property_name = input_parts[0].trim_ascii().trim_matches(&['"', ','][..]);
//     let mut property_values: Vec<&str> = Vec::new();
//     if input_parts.len() > 1 {
//         property_values = input_parts[1]
//             .trim_matches(']')
//             .split(", ")
//             .map(|p| p.trim_ascii().trim_matches(&['"', ','][..]))
//             .collect::<Vec<&str>>();
//     }

//     let item: proc_macro2::TokenStream = item.into();

//     let (variants, is_enum): (Vec<proc_macro2::Ident>, bool) = match ast.data {
//         syn::Data::Enum(enum_item) => (
//             enum_item.variants.into_iter().map(|v| v.ident).collect(),
//             true,
//         ),
//         syn::Data::Struct(s) => {
//             let fields = match s.fields {
//                 Fields::Named(f) => abort!(f.span(), "Block properties can't have named fields"),
//                 Fields::Unnamed(fields) => fields.unnamed,
//                 Fields::Unit => abort!(s.fields.span(), "Block properties must have fields"),
//             };
//             if fields.len() != 1 {
//                 abort!(
//                     fields.span(),
//                     "Block properties `struct`s must have exactly one field"
//                 );
//             }
//             let field = fields.first().unwrap();
//             let ty = &field.ty;
//             let struct_type = match field.ty {
//                 syn::Type::Path(ref type_path) => {
//                     type_path.path.segments.first().unwrap().ident.to_string()
//                 }
//                 ref other => abort!(
//                     other.span(),
//                     "Block properties can only have primitive types"
//                 ),
//             };
//             match struct_type.as_str() {
//                 "bool" => (
//                     vec![
//                         proc_macro2::Ident::new("true", proc_macro2::Span::call_site()),
//                         proc_macro2::Ident::new("false", proc_macro2::Span::call_site()),
//                     ],
//                     false,
//                 ),
//                 other => abort!(
//                     ty.span(),
//                     format!("`{other}` is not supported (why not implement it yourself?)")
//                 ),
//             }
//         }
//         _ => abort_call_site!("Block properties can only be `enum`s or `struct`s"),
//     };

//     let values = variants.iter().enumerate().map(|(i, v)| match is_enum {
//         true => {
//             let mut value = v.to_string().to_snake_case();
//             if !property_values.is_empty() && i < property_values.len() {
//                 value = property_values[i].to_string();
//             }
//             quote! {
//                 Self::#v => #value.to_string(),
//             }
//         }
//         false => {
//             let value = v.to_string();
//             quote! {
//                 Self(#v) => #value.to_string(),
//             }
//         }
//     });

//     let from_values = variants.iter().enumerate().map(|(i, v)| match is_enum {
//         true => {
//             let mut value = v.to_string().to_snake_case();
//             if !property_values.is_empty() && i < property_values.len() {
//                 value = property_values[i].to_string();
//             }
//             quote! {
//                 #value => Self::#v,
//             }
//         }
//         false => {
//             let value = v.to_string();
//             quote! {
//                 #value => Self(#v),
//             }
//         }
//     });

//     let extra_fns = variants.iter().map(|v| {
//         let title = proc_macro2::Ident::new(
//             &v.to_string().to_pascal_case(),
//             proc_macro2::Span::call_site(),
//         );
//         quote! {
//             pub fn #title() -> Self {
//                 Self(#v)
//             }
//         }
//     });

//     let extra = if is_enum {
//         quote! {}
//     } else {
//         quote! {
//             impl #name {
//                 #(#extra_fns)*
//             }
//         }
//     };

//     let code = quote! {
//         #item
//         impl #impl_generics pumpkin_world::block::properties::BlockPropertyMetadata for #name #ty_generics {
//             fn name(&self) -> &'static str {
//                 #property_name
//             }
//             fn value(&self) -> String {
//                 match self {
//                     #(#values)*
//                 }
//             }
//             fn from_value(value: String) -> Self {
//                 match value.as_str() {
//                     #(#from_values)*
//                     _ => panic!("Invalid value for block property"),
//                 }
//             }
//         }
//         #extra
//     };

//     code.into()
// }

#[rustfmt::skip]
#[proc_macro_derive(PacketWrite, attributes(serial))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data) = &input.data {
        data.fields.iter().map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let (is_big_endian, no_prefix) = check_serial_attributes(&f.attrs);
            let is_vec = is_vec(&f.ty);

            if is_vec && !no_prefix {
                // Vec with prefix: write VarUInt length, then data
                if is_big_endian {
                    quote! {
                        crate::codec::var_uint::VarUInt(self.#ident.len() as u32).write(writer)?;
                        self.#ident.write_be(writer)?;
                    }
                } else {
                    quote! {
                        crate::codec::var_uint::VarUInt(self.#ident.len() as u32).write(writer)?;
                        self.#ident.write(writer)?;
                    }
                }
            } else {
                // Non-Vec or Vec with no_prefix: write directly
                if is_big_endian {
                    quote! {
                        self.#ident.write_be(writer)?;
                    }
                } else {
                    quote! {
                        self.#ident.write(writer)?;
                    }
                }
            }
        })
    } else {
        unimplemented!()
    };

    let expanded = quote! {
        impl PacketWrite for #name {
            fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
                #(#fields)*
                Ok(())
            }
        }
    };

    expanded.into()
}

#[rustfmt::skip]
#[proc_macro_derive(PacketRead, attributes(serial))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data) = &input.data {
        data.fields.iter().map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let (is_big_endian, no_prefix) = check_serial_attributes(&f.attrs);
            let is_vec = is_vec(&f.ty);

            if is_vec && !no_prefix {
                // Vec with prefix: read VarUInt length, then data
                quote! {
                    #ident: {
                        let len = crate::codec::var_uint::VarUInt::read(reader)?.0 as usize;
                        let mut buf = vec![0u8; len];
                        reader.read_exact(&mut buf)?;
                        buf
                    }
                }
            } else {
                // Non-Vec or Vec with no_prefix: read directly
                if is_big_endian {
                    quote! {
                        #ident: PacketRead::read_be(reader)?
                    }
                } else {
                    quote! {
                        #ident: PacketRead::read(reader)?
                    }
                }
            }
        })
    } else {
        unimplemented!()
    };

    let expanded = quote! {
        impl PacketRead for #name {
            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    };

    expanded.into()
}

fn check_serial_attributes(attrs: &[Attribute]) -> (bool, bool) {
    let mut is_big_endian = false;
    let mut no_prefix = false;

    for attr in attrs.iter() {
        if attr.path().is_ident("serial") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("big_endian") {
                    is_big_endian = true;
                } else if meta.path.is_ident("no_prefix") {
                    no_prefix = true;
                }
                Ok(())
            });
        }
    }

    (is_big_endian, no_prefix)
}

fn is_vec(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .iter()
            .last()
            .map(|segment| segment.ident == "Vec")
            .unwrap_or(false)
    } else {
        false
    }
}
