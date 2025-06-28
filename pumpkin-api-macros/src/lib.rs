use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use proc_macro2::Ident;
use quote::quote;
use std::sync::LazyLock;
use std::sync::Mutex;
use syn::{ImplItem, ItemFn, ItemImpl, ItemStruct, parse_macro_input, parse_quote};

static PLUGIN_METHODS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[proc_macro_error]
#[proc_macro_attribute]
pub fn plugin_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    let method = quote! {
        #[allow(unused_mut)]
        async fn #fn_name(#fn_inputs) #fn_output {
            crate::GLOBAL_RUNTIME.block_on(async move {
                #fn_body
            })
        }
    }
    .to_string();

    PLUGIN_METHODS.lock().unwrap().push(method);

    TokenStream::new()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn plugin_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = &input_struct.ident;

    let methods = PLUGIN_METHODS.lock().unwrap();

    let methods: Vec<proc_macro2::TokenStream> = methods
        .iter()
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
        pub static GLOBAL_RUNTIME: std::sync::LazyLock<std::sync::Arc<tokio::runtime::Runtime>> =
            std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()));

        #[unsafe(no_mangle)]
        pub static METADATA: pumpkin::plugin::PluginMetadata = pumpkin::plugin::PluginMetadata {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            authors: env!("CARGO_PKG_AUTHORS"),
            description: env!("CARGO_PKG_DESCRIPTION"),
        };

        #input_struct

        #[async_trait::async_trait]
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
