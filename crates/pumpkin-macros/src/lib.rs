#![allow(clippy::unwrap_used, clippy::expect_used)]

use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use proc_macro_error2::{abort, abort_call_site};
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::{Block, BlockId};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{self, Attribute, DeriveInput, LitStr, Type, parse_quote};
use syn::{Block as SynBlock, Expr, Field, Fields, ItemStruct, Stmt, parse_macro_input};

/// Derives the `Payload` trait for an event struct, enabling it to be used in the plugin system.
///
/// # Arguments
/// - `item` – The input `TokenStream` representing the struct to derive `Event` for.
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

/// Marks a struct as cancellable by adding a `cancelled: bool` field and
/// implementing the `Cancellable` trait.
///
/// # Arguments
/// - `_args` – `TokenStream` of arguments passed to the attribute (unused).
/// - `input` – The input `TokenStream` representing the struct to modify.
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
                .any(|f| f.ident.as_ref().is_some_and(|i| i == "cancelled"))
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

/// Sends a cancellable event through the plugin manager.
///
/// # Syntax
/// ```ignore
/// send_cancellable! {{
///     <server_expr>;
///     <event_expr>;
///     'after: { <after_stmts> }
///     'cancelled: { <cancelled_stmts> }
/// }}
/// ```
#[proc_macro]
pub fn send_cancellable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SynBlock);

    let mut stmts_iter = input.stmts.into_iter();

    let Some(Stmt::Expr(server_stmt, _)) = stmts_iter.next() else {
        abort_call_site!("expected server expression as first statement")
    };

    let Some(Stmt::Expr(event_stmt, _)) = stmts_iter.next() else {
        abort_call_site!("expected event expression as second statement")
    };

    let event_expr = if let Expr::Reference(syn::ExprReference {
        expr,
        mutability: Some(_),
        ..
    }) = event_stmt
    {
        *expr
    } else {
        event_stmt
    };

    let mut after_block = None;
    let mut cancelled_block = None;

    for stmt in stmts_iter {
        if let Stmt::Expr(Expr::Block(b), _) = stmt
            && let Some(ref label) = b.label
        {
            if label.name.ident == "after" {
                after_block = Some(b.block);
            } else if label.name.ident == "cancelled" {
                cancelled_block = Some(b.block);
            }
        }
    }

    let execution = match (after_block, cancelled_block) {
        (Some(after), Some(cancelled)) => quote! {
            if !is_cancelled {
                #after
            } else {
                #cancelled
            }
        },
        (Some(after), None) => quote! {
            if !is_cancelled {
                #after
            }
        },
        (None, Some(cancelled)) => quote! {
            if is_cancelled {
                #cancelled
            }
        },
        (None, None) => quote! {},
    };

    let expanded = quote! {
        {
            let mut event = #event_expr;
            let server_ref: &std::sync::Arc<crate::server::Server> = {
                use std::borrow::Borrow;
                (#server_stmt).borrow()
            };
            server_ref.plugin_manager.fire(server_ref, &mut event).await;

            let is_cancelled = {
                use crate::plugin::Cancellable;
                event.cancelled()
            };

            #execution
        }
    };

    expanded.into()
}

/// Attaches a fixed packet ID to a struct implementing `Packet`.
///
/// # Arguments
/// - `args` – The `TokenStream` representing the packet ID expression.
/// - `item` – The input `TokenStream` representing the struct to implement `Packet` for.
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

/// Attaches a multi-version packet ID to a struct implementing `MultiVersionJavaPacket`.
///
/// # Arguments
/// - `args` – The `TokenStream` representing the packet ID expression.
/// - `item` – The input `TokenStream` representing the struct to implement the trait for.
#[proc_macro_attribute]
pub fn java_packet(args: TokenStream, item: TokenStream) -> TokenStream {
    let packet_id_expr = parse_macro_input!(args as Expr);
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        #ast
        impl #impl_generics crate::packet::MultiVersionJavaPacket for #name #ty_generics #where_clause {
            #[must_use]
            #[inline]
            fn to_id(version: pumpkin_util::version::JavaMinecraftVersion) -> i32 {
                #packet_id_expr.to_id(version)
            }
        }
    }
    .into()
}

/// Marks a struct as representing a specific block by its name.
///
/// # Arguments
/// - `args` – The `TokenStream` representing the block name literal.
/// - `item` – The input `TokenStream` representing the struct to implement `BlockMetadata` for.
#[proc_macro_attribute]
pub fn pumpkin_block(args: TokenStream, item: TokenStream) -> TokenStream {
    let input_item = item.clone();

    let arg_lit = parse_macro_input!(args as LitStr);
    let arg_value = arg_lit.value();

    let block_name = arg_value.strip_prefix("minecraft:").unwrap_or(&arg_value);
    let Some(block) = Block::from_name(block_name) else {
        return syn::Error::new(arg_lit.span(), "Invalid block name")
            .to_compile_error()
            .into();
    };
    let const_ident = format_ident!("{}", block.name.to_shouty_snake_case());

    let ast = parse_macro_input!(item as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let generated = quote! {
        impl #impl_generics crate::block::BlockMetadata for #name #ty_generics #where_clause {
            fn ids() -> Box<[pumpkin_data::BlockId]> {
                [pumpkin_data::BlockId::#const_ident].into()
            }
        }
    };

    // Combine the original item and new impl.
    let mut output = input_item;
    output.extend(TokenStream::from(generated));
    output
}

/// Marks a struct as representing a set of blocks from a given tag.
///
/// # Arguments
/// - `args` – The `TokenStream` representing the block tag literal.
/// - `item` – The input `TokenStream` representing the struct to implement `BlockMetadata` for.
#[proc_macro_attribute]
pub fn pumpkin_block_from_tag(args: TokenStream, item: TokenStream) -> TokenStream {
    let original_item = item.clone();

    let arg_lit = parse_macro_input!(args as LitStr);
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let full_tag = arg_lit.value();

    let Some(values) = get_tag_ids(RegistryKey::Block, &full_tag) else {
        return syn::Error::new(arg_lit.span(), format!("Failed to get tag IDs: {full_tag}"))
            .to_compile_error()
            .into();
    };
    let const_values: Vec<_> = values
        .iter()
        .map(|v| {
            let block = BlockId::new_or_air(*v).to_block();
            format_ident!("{}", block.name.to_shouty_snake_case())
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics crate::block::BlockMetadata for #name #ty_generics #where_clause {
            fn ids() -> Box<[pumpkin_data::BlockId]> {
                Box::new([ #(pumpkin_data::BlockId::#const_values),* ])
            }
        }
    };

    let mut output = original_item;
    output.extend(TokenStream::from(expanded));
    output
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
//                     "Block properties `struct's must have exactly one field"
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
//         _ => abort_call_site!("Block properties can only be `enum`s or `struct's"),
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

/// Derives the `PacketWrite` trait for a struct, enabling serialization.
///
/// # Arguments
/// - `input` – The input `TokenStream` representing the struct to derive `PacketWrite` for.
#[rustfmt::skip]
#[proc_macro_derive(PacketWrite, attributes(serial))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
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
        return syn::Error::new(name.span(), "Only structs are supported")
            .to_compile_error()
            .into();
    };

    let type_generic = match input.generics.params.len() {
        0 => quote! {},
        1 => quote! { <'_> },
        _ => {
            return syn::Error::new(name.span(), "Only up to one lifetime parameter is supported.")
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        impl PacketWrite for #name #type_generic {
            fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
                #(#fields)*
                Ok(())
            }
        }
    };

    expanded.into()
}

/// Derives the `PacketRead` trait for a struct, enabling deserialization.
///
/// # Arguments
/// - `input` – The input `TokenStream` representing the struct to derive `PacketRead` for.
#[rustfmt::skip]
#[proc_macro_derive(PacketRead, attributes(serial))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data) = &input.data {
        data.fields.iter().map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let (is_big_endian, no_prefix) = check_serial_attributes(&f.attrs);
            let is_vec = is_vec(&f.ty);

            if is_vec && no_prefix {
                return syn::Error::new(name.span(), "Cannot handle non-prefixed vecs")
                    .to_compile_error();
            }

            // Non-Vec or Vec without no_prefix: read directly
            if is_big_endian {
                quote! {
                    #ident: PacketRead::read_be(reader)?
                }
            } else {
                quote! {
                    #ident: PacketRead::read(reader)?
                }
            }
        })
    } else {
        return syn::Error::new(name.span(), "Only structs are supported")
            .to_compile_error()
            .into();
    };


    let type_generic = match input.generics.params.len() {
        0 => quote! {},
        1 => quote! { <'static> },
        _ => {
            return syn::Error::new(name.span(), "Only up to one lifetime parameter is supported.")
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        impl PacketRead for #name #type_generic {
            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    };

    expanded.into()
}

/// Derives the `PacketReadSlice` trait for a struct, enabling deserialization from a slice.
///
/// # Arguments
/// - `input` – The input `TokenStream` representing the struct to derive `PacketReadSlice` for.
#[rustfmt::skip]
#[proc_macro_derive(PacketReadSlice, attributes(serial))]
pub fn derive_deserialize_from_slice(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data) = &input.data {
        data.fields.iter().map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let (is_big_endian, no_prefix) = check_serial_attributes(&f.attrs);
            let is_vec = is_vec(&f.ty);

            if is_vec && no_prefix {
                return syn::Error::new(name.span(), "Cannot handle non-prefixed vecs.")
                    .to_compile_error();
            }

            // Non-Vec or Vec without no_prefix: read directly
            if is_big_endian {
                return syn::Error::new(name.span(), "Cannot handle big-endian encoded fields")
                    .to_compile_error();
            }

            quote! {
                #ident: PacketReadSlice::read_slice(buf)?
            }
        })
    } else {
        return syn::Error::new(name.span(), "Only structs are supported")
            .to_compile_error()
            .into();
    };

    let expanded = quote! {
        impl<'a> PacketReadSlice<'a> for #name<'a> {
            fn read_slice(buf: &mut &'a [u8]) -> std::io::Result<Self> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    };

    expanded.into()
}

/// Checks a field's `#[serial(...)]` attributes.
///
/// # Arguments
/// - `attrs` – Slice of `Attribute`s to inspect for serial-specific metadata.
///
/// # Returns
/// Tuple `(is_big_endian, no_prefix)` indicating whether the field is big-endian
/// and/or has no length prefix.
fn check_serial_attributes(attrs: &[Attribute]) -> (bool, bool) {
    let mut is_big_endian = false;
    let mut no_prefix = false;

    for attr in attrs {
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

/// Returns true if the type is a `Vec<_>`.
///
/// # Arguments
/// - `ty` – The `Type` to check.
///
/// # Returns
/// `true` if the type is a `Vec`, otherwise `false`.
fn is_vec(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .iter()
            .next_back()
            .is_some_and(|segment| segment.ident == "Vec")
    } else {
        false
    }
}

struct TranslateCrossInput {
    java_expr: syn::Expr,
    bedrock_expr: syn::Expr,
    args: Vec<syn::Expr>,
}

impl syn::parse::Parse for TranslateCrossInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let java_expr: syn::Expr = input.parse()?;
        let _ = input.parse::<syn::Token![,]>()?;
        let bedrock_expr: syn::Expr = input.parse()?;

        let mut args = Vec::new();
        while !input.is_empty() {
            let _ = input.parse::<syn::Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse()?);
        }

        Ok(Self {
            java_expr,
            bedrock_expr,
            args,
        })
    }
}

fn eval_translation_key_expr(expr: &syn::Expr) -> Option<(&'static str, proc_macro2::Span)> {
    match expr {
        syn::Expr::Path(expr_path) => {
            let segments: Vec<String> = expr_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            let seg_refs: Vec<&str> = segments.iter().map(String::as_str).collect();

            let (is_java, const_ident) = match seg_refs.as_slice() {
                ["translation", "java", ident]
                | ["pumpkin_data" | "crate", "translation", "java", ident] => (true, *ident),
                ["translation", "bedrock", ident]
                | ["pumpkin_data" | "crate", "translation", "bedrock", ident] => (false, *ident),
                _ => return None,
            };

            let key = if is_java {
                pumpkin_data::translation::java::get(const_ident)
                    .and_then(pumpkin_data::translation::java::get_value)
            } else {
                pumpkin_data::translation::bedrock::get(const_ident)
                    .and_then(pumpkin_data::translation::bedrock::get_value)
            };

            key.map(|k| (k, expr.span()))
        }
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => {
            // Leaking string slice here is fine since this runs during compilation inside proc macro
            let s = Box::leak(lit_str.value().into_boxed_str());
            Some((s, lit_str.span()))
        }
        _ => None,
    }
}

fn count_placeholders(format_str: &str) -> usize {
    let mut count = 0;
    let bytes = format_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'%' {
                i += 2;
                continue;
            }
            // Handle positional specifiers like %1$s, %2$d
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'$' {
                count += 1;
                i = j + 2; // skip past specifier character following $ (e.g., $s)
                continue;
            }
            // Handle optional width/flags before specifier type like %s, %d, %f, %1s
            count += 1;
            i = j + 1; // skip past specifier character
            continue;
        }
        i += 1;
    }
    count
}

/// Validates translation keys and arguments at compile-time and returns a `TextComponent`.
#[proc_macro]
pub fn translate_cross(input: TokenStream) -> TokenStream {
    let TranslateCrossInput {
        java_expr,
        bedrock_expr,
        args,
    } = parse_macro_input!(input as TranslateCrossInput);

    if let Some((java_str, span)) = eval_translation_key_expr(&java_expr) {
        let expected = count_placeholders(java_str);
        // Java translation keys from generated constants might differ from Bedrock key placeholder formats
        if expected != args.len() && matches!(java_expr, syn::Expr::Lit(_)) {
            return syn::Error::new(
                span,
                format!(
                    "Java translation key `{}` expects {} argument(s), but {} were provided",
                    java_str,
                    expected,
                    args.len()
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    if let Some((bedrock_str, span)) = eval_translation_key_expr(&bedrock_expr) {
        let expected = count_placeholders(bedrock_str);
        if expected != args.len() && matches!(bedrock_expr, syn::Expr::Lit(_)) {
            return syn::Error::new(
                span,
                format!(
                    "Bedrock translation key `{}` expects {} argument(s), but {} were provided",
                    bedrock_str,
                    expected,
                    args.len()
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let expanded = quote! {
        {
            #[allow(deprecated)]
            pumpkin_util::text::TextComponent::translate_cross(#java_expr, #bedrock_expr, vec![#(#args),*])
        }
    };

    expanded.into()
}

struct TranslateJavaInput {
    java_expr: syn::Expr,
    args: Vec<syn::Expr>,
}

impl syn::parse::Parse for TranslateJavaInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let java_expr: syn::Expr = input.parse()?;

        let mut args = Vec::new();
        while !input.is_empty() {
            let _ = input.parse::<syn::Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse()?);
        }

        Ok(Self { java_expr, args })
    }
}

/// Validates Java translation keys and arguments at compile-time and returns a `TextComponent`.
#[deprecated(
    since = "0.1.0",
    note = "Use `pumpkin_macros::translate_cross!` macro instead for cross-platform (Java & Bedrock) translation support."
)]
#[proc_macro]
pub fn translate_java(input: TokenStream) -> TokenStream {
    let TranslateJavaInput { java_expr, args } = parse_macro_input!(input as TranslateJavaInput);

    if let Some((java_str, span)) = eval_translation_key_expr(&java_expr) {
        let expected = count_placeholders(java_str);
        if expected != args.len() {
            return syn::Error::new(
                span,
                format!(
                    "Java translation key `{}` expects {} argument(s), but {} were provided",
                    java_str,
                    expected,
                    args.len()
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let expanded = quote! {
        {
            #[allow(deprecated)]
            pumpkin_util::text::TextComponent::translate(#java_expr, vec![#(#args),*])
        }
    };

    expanded.into()
}
