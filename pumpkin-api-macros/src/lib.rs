use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use proc_macro2::Ident;
use quote::quote;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use syn::{ImplItem, ItemFn, ItemImpl, ItemStruct, parse_macro_input, parse_quote};

/// Stores all functions annotated with `#[plugin_method]` for later use in the impl.
static PLUGIN_METHODS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Marks a function as a plugin method, wrapping it to return a `PluginFuture`.
///
/// The function body will be executed asynchronously using the global runtime.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn plugin_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    let output_type = match fn_output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    let method = quote! {
        #[expect(unused_mut)]
        fn #fn_name(#fn_inputs) -> PluginFuture<'_, #output_type> {
            crate::GLOBAL_RUNTIME.block_on(async move {
                Box::pin(async move {
                    #fn_body
                })
            })
        }
    }
    .to_string();

    PLUGIN_METHODS
        .lock()
        .unwrap()
        .insert(fn_name.to_string(), method);

    TokenStream::new()
}

/// Wraps a struct as a plugin implementation.
///
/// This generates the `Plugin` impl for the struct, including all `#[plugin_method]` annotated functions,
/// and provides a `plugin()` function to return it as a boxed trait object.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn plugin_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = &input_struct.ident;

    let methods = PLUGIN_METHODS.lock().unwrap();

    let methods: Vec<proc_macro2::TokenStream> = methods
        .values()
        .map(|s| {
            s.parse::<proc_macro2::TokenStream>().unwrap_or_else(|e| {
                abort!(
                    struct_ident,
                    format!("re-parsing cached method failed: {e}")
                )
            })
        })
        .collect();

    // Combine the original struct definition with the impl block and plugin() function
    let expanded = quote! {
        use pumpkin::plugin::PluginFuture;

        pub static GLOBAL_RUNTIME: std::sync::LazyLock<std::sync::Arc<tokio::runtime::Runtime>> =
            std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()));

        #[unsafe(no_mangle)]
        pub static METADATA: std::sync::LazyLock<pumpkin::plugin::PluginMetadata> = std::sync::LazyLock::new(|| {
            pumpkin::plugin::PluginMetadata {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                authors: env!("CARGO_PKG_AUTHORS").split(',').map(String::from).collect(),
                description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            }
        });

        #[unsafe(no_mangle)]
        pub static PUMPKIN_API_VERSION: u32 = pumpkin::plugin::PLUGIN_API_VERSION;

        #input_struct

        impl pumpkin::plugin::Plugin for #struct_ident {
            #(#methods)*
        }

        #[unsafe(no_mangle)]
        pub fn plugin() -> Box<dyn pumpkin::plugin::Plugin> {
            Box::new(#struct_ident::new())
        }
    };

    TokenStream::from(expanded)
}

/// Wraps all functions in an `impl` block to execute with a runtime.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn with_runtime(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let mode: Ident = parse_macro_input!(attr as Ident);
    let use_global = match mode.to_string().as_str() {
        "global" => true,
        "local" => false,
        other => abort!(mode, format!("expected `global` or `local`, got `{other}`")),
    };

    for item in &mut input.items {
        if let ImplItem::Fn(method) = item {
            let original_body = &method.block;

            method.block = if use_global {
                parse_quote!({
                    crate::GLOBAL_RUNTIME.block_on(async move {
                        #original_body
                    })
                })
            } else {
                parse_quote!({
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async move {
                            #original_body
                        })
                })
            };
        }
    }

    TokenStream::from(quote!(#input))
}
