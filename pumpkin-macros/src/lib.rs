use heck::{ToPascalCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro_error2::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{self, DeriveInput};
use syn::{
    Block, Expr, Field, Fields, ItemStruct, Stmt,
    parse::{Nothing, Parser},
    parse_macro_input,
};

extern crate proc_macro;

#[proc_macro_derive(Event)]
pub fn event(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    quote! {
        impl crate::plugin::Event for #name {
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
pub fn cancellable(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let name = item_struct.ident.clone();
    let _ = parse_macro_input!(args as Nothing);

    if let Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            Field::parse_named
                .parse2(quote! {
                    /// A boolean indicating cancel state of the event.
                    pub cancelled: bool
                })
                .unwrap(),
        );
    }

    quote! {
        #item_struct

        impl crate::plugin::Cancellable for #name {
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
    let input = parse_macro_input!(input as Block);

    let mut event = None;
    let mut after_block = None;
    let mut cancelled_block = None;

    for stmt in input.stmts {
        if let Stmt::Expr(expr, _) = stmt {
            if event.is_none() {
                event = Some(expr);
            } else if let Expr::Block(b) = expr {
                if let Some(ref label) = b.label {
                    if label.name.ident == "after" {
                        after_block = Some(b);
                    } else if label.name.ident == "cancelled" {
                        cancelled_block = Some(b);
                    }
                }
            }
        }
    }

    if let Some(event) = event {
        if let Some(after_block) = after_block {
            if let Some(cancelled_block) = cancelled_block {
                quote! {
                    let event = crate::PLUGIN_MANAGER
                        .read()
                        .await
                        .fire(#event)
                        .await;

                    if !event.cancelled {
                        #after_block
                    } else {
                        #cancelled_block
                    }
                }
                .into()
            } else {
                quote! {
                    let event = crate::PLUGIN_MANAGER
                        .read()
                        .await
                        .fire(#event)
                        .await;

                    if !event.cancelled {
                        #after_block
                    }
                }
                .into()
            }
        } else if let Some(cancelled_block) = cancelled_block {
            quote! {
                let event = crate::PLUGIN_MANAGER
                    .read()
                    .await
                    .fire(#event)
                    .await;

                if event.cancelled {
                    #cancelled_block
                }
            }
            .into()
        } else {
            quote! {
                let event = crate::PLUGIN_MANAGER
                    .read()
                    .await
                    .fire(#event)
                    .await;
            }
            .into()
        }
    } else {
        abort_call_site!("Event must be specified");
    }
}

#[proc_macro_attribute]
pub fn packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = input.into();
    let item: proc_macro2::TokenStream = item.into();

    let code = quote! {
        #item
        impl #impl_generics crate::packet::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input;
        }
    };

    code.into()
}

#[proc_macro_attribute]
pub fn pumpkin_block(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input_string = input.to_string();
    let packet_name = input_string.trim_matches('"');
    let packet_name_split: Vec<&str> = packet_name.split(":").collect();

    let namespace = packet_name_split[0];
    let id = packet_name_split[1];

    let item: proc_macro2::TokenStream = item.into();

    let code = quote! {
        #item
        impl #impl_generics crate::block::pumpkin_block::BlockMetadata for #name #ty_generics {
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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn block_property(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input_string = input.to_string();
    let input_parts: Vec<&str> = input_string.split("[").collect();
    let property_name = input_parts[0].trim_ascii().trim_matches(&['"', ','][..]);
    let mut property_values: Vec<&str> = Vec::new();
    if input_parts.len() > 1 {
        property_values = input_parts[1]
            .trim_matches(']')
            .split(", ")
            .map(|p| p.trim_ascii().trim_matches(&['"', ','][..]))
            .collect::<Vec<&str>>();
    }

    let item: proc_macro2::TokenStream = item.into();

    let (variants, is_enum): (Vec<proc_macro2::Ident>, bool) = match ast.data {
        syn::Data::Enum(enum_item) => (
            enum_item.variants.into_iter().map(|v| v.ident).collect(),
            true,
        ),
        syn::Data::Struct(s) => {
            let fields = match s.fields {
                Fields::Named(f) => abort!(f.span(), "Block properties can't have named fields"),
                Fields::Unnamed(fields) => fields.unnamed,
                Fields::Unit => abort!(s.fields.span(), "Block properties must have fields"),
            };
            if fields.len() != 1 {
                abort!(
                    fields.span(),
                    "Block properties `struct`s must have exactly one field"
                );
            }
            let field = fields.first().unwrap();
            let ty = &field.ty;
            let struct_type = match field.ty {
                syn::Type::Path(ref type_path) => {
                    type_path.path.segments.first().unwrap().ident.to_string()
                }
                ref other => abort!(
                    other.span(),
                    "Block properties can only have primitive types"
                ),
            };
            match struct_type.as_str() {
                "bool" => (
                    vec![
                        proc_macro2::Ident::new("true", proc_macro2::Span::call_site()),
                        proc_macro2::Ident::new("false", proc_macro2::Span::call_site()),
                    ],
                    false,
                ),
                other => abort!(
                    ty.span(),
                    format!("`{other}` is not supported (why not implement it yourself?)")
                ),
            }
        }
        _ => abort_call_site!("Block properties can only be `enum`s or `struct`s"),
    };

    let values = variants.iter().enumerate().map(|(i, v)| match is_enum {
        true => {
            let mut value = v.to_string().to_snake_case();
            if !property_values.is_empty() && i < property_values.len() {
                value = property_values[i].to_string();
            }
            quote! {
                Self::#v => #value.to_string(),
            }
        }
        false => {
            let value = v.to_string();
            quote! {
                Self(#v) => #value.to_string(),
            }
        }
    });

    let from_values = variants.iter().enumerate().map(|(i, v)| match is_enum {
        true => {
            let mut value = v.to_string().to_snake_case();
            if !property_values.is_empty() && i < property_values.len() {
                value = property_values[i].to_string();
            }
            quote! {
                #value => Self::#v,
            }
        }
        false => {
            let value = v.to_string();
            quote! {
                #value => Self(#v),
            }
        }
    });

    let extra_fns = variants.iter().map(|v| {
        let title = proc_macro2::Ident::new(
            &v.to_string().to_pascal_case(),
            proc_macro2::Span::call_site(),
        );
        quote! {
            pub fn #title() -> Self {
                Self(#v)
            }
        }
    });

    let extra = if is_enum {
        quote! {}
    } else {
        quote! {
            impl #name {
                #(#extra_fns)*
            }
        }
    };

    let code = quote! {
        #item
        impl #impl_generics pumpkin_world::block::properties::BlockPropertyMetadata for #name #ty_generics {
            fn name(&self) -> &'static str {
                #property_name
            }
            fn value(&self) -> String {
                match self {
                    #(#values)*
                }
            }
            fn from_value(value: String) -> Self {
                match value.as_str() {
                    #(#from_values)*
                    _ => panic!("Invalid value for block property"),
                }
            }
        }
        #extra
    };

    code.into()
}

#[proc_macro_derive(PersistentDataHolder, attributes(persistent_data))]
/// Derive macro to automatically implement the `PersistentDataHolder` trait for a struct.
///
/// This macro looks for a single struct field annotated with `#[persistent_data]`
/// and verifies that this field is of type `PersistentDataContainer`.
///
/// It then generates the implementation of the `PersistentDataHolder` trait
/// that delegates all trait methods to this field.
///
/// # Requirements
/// - Exactly one struct field must be annotated with `#[persistent_data]`.
/// - The annotated field must have the type `PersistentDataContainer` (fully qualified as needed).
///
/// # Errors
/// - Compilation will fail if no field is annotated with `#[persistent_data]`.
/// - Compilation will fail if the annotated field is not of the correct type.
///
/// # Example
/// ```ignore
/// use your_crate::PersistentDataHolder;
///
/// // Automatically implements PersistentDataHolder for `Person`
/// #[derive(PersistentDataHolder)]
/// pub struct Person {
///     pub name: String,
///     #[persistent_data]
///     pub(crate) container: PersistentDataContainer,
/// }
/// ```
pub fn derive_persistent(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Find the struct field annotated with #[persistent_data]
    let Some((field_ident, field_ty)) = (if let syn::Data::Struct(data) = &input.data {
        data.fields.iter().find_map(|f| {
            for attr in &f.attrs {
                if attr.path().is_ident("persistent_data") {
                    return Some((f.ident.clone()?, &f.ty));
                }
            }
            None
        })
    } else {
        None
    }) else {
        return syn::Error::new_spanned(
            name,
            "No field annotated with #[persistent_data] found in struct",
        )
        .to_compile_error()
        .into();
    };

    // Verify the type of the annotated field is PersistentDataContainer
    let is_valid_type = match field_ty {
        syn::Type::Path(type_path) => {
            let segments: Vec<_> = type_path.path.segments.iter().collect();
            match segments.as_slice() {
                // Handles local type: PersistentDataContainer
                [seg] => seg.ident == "PersistentDataContainer",
                // Handles fully qualified type path, e.g. pumpkin::plugin::api::persistence::PersistentDataContainer
                [a, b, c, d, e] => {
                    a.ident == "pumpkin"
                        && b.ident == "plugin"
                        && c.ident == "api"
                        && d.ident == "persistence"
                        && e.ident == "PersistentDataContainer"
                }
                _ => false,
            }
        }
        _ => false,
    };

    if !is_valid_type {
        return syn::Error::new_spanned(
            field_ty,
            "Field annotated with #[persistent_data] must have type `PersistentDataContainer`",
        )
        .to_compile_error()
        .into();
    }

    let field = field_ident;

    // Generate the implementation of PersistentDataHolder trait by delegating to the annotated field
    let expanded = quote! {
        impl PersistentDataHolder for #name {
            fn clear(&self) {
                self.#field.clear();
            }

            fn get(&self, key: &NamespacedKey) -> Option<PersistentDataType> {
                self.#field.get(key).map(|v| v.clone())
            }

            fn get_as<T: FromPersistentDataType>(&self, key: &NamespacedKey) -> Option<T> {
                self.get(key).and_then(|v| T::from_persistent(&v))
            }

            fn insert(&self, key: &NamespacedKey, value: PersistentDataType) {
                if let Some(_old_value) = self.#field.insert(key.clone(), value) {
                    #[cfg(debug_assertions)]
                    log::warn!("Inserting key {:?} which already existed in PersistentDataContainer, overwriting old value.", key);
                }
            }

            fn remove(&self, key: &NamespacedKey) -> Option<PersistentDataType> {
                self.#field.remove(key).map(|(_k, v)| v)
            }

            fn contains_key(&self, key: &NamespacedKey) -> bool {
                self.#field.contains_key(key)
            }

            fn iter(&self) -> Box<dyn Iterator<Item = (NamespacedKey, PersistentDataType)> + '_> {
                Box::new(self.#field.iter().map(|entry| (entry.key().clone(), entry.value().clone())))
            }

            fn container_mut(&mut self) -> &mut PersistentDataContainer {
                &mut self.#field
            }
        }
    };

    TokenStream::from(expanded)
}
