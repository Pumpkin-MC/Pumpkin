//! Proc-macro entry point for generating Pumpkin native WIT host bindings.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitBool, LitStr, Result, Token, braced};

mod abi_gen;
mod codegen;
mod rust;
mod symbol_name;
mod types;

use codegen::{ImportMode, Opts};

#[proc_macro]
pub fn bindgen(input: TokenStream) -> TokenStream {
    match bindgen_impl(input.into()) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn bindgen_impl(input: TokenStream2) -> Result<TokenStream2> {
    let config: BindgenConfig = syn::parse2(input)?;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let wit_path = PathBuf::from(&manifest_dir).join(&config.path);

    let mut resolve = wit_parser::Resolve::new();
    let (pkg, sources) = resolve.push_path(&wit_path).map_err(|err| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("failed to load WIT from `{}`: {err}", wit_path.display()),
        )
    })?;
    let wit_files: Vec<PathBuf> = sources.paths().map(Path::to_path_buf).collect();

    let world_id = resolve
        .select_world(&[pkg], Some(&config.world))
        .map_err(|err| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("failed to select WIT world `{}`: {err}", config.world),
            )
        })?;

    let opts = Opts {
        store_data: config.store_data,
        concurrent: config.concurrent,
        default_import: config.default_import,
        imports: config.imports,
    };

    let mut code = opts
        .generate(&mut resolve, world_id)
        .map_err(|err| syn::Error::new(proc_macro2::Span::call_site(), format!("{err}")))?;

    for file in &wit_files {
        let _ = writeln!(
            code,
            "const _: &[u8] = include_bytes!(r#\"{}\"#);",
            file.display()
        );
    }

    code.parse::<TokenStream2>().map_err(|err| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("pumpkin-native-wit-macro generated invalid Rust tokens: {err}"),
        )
    })
}

struct BindgenConfig {
    path: String,
    world: String,
    store_data: String,
    concurrent: bool,
    default_import: ImportMode,
    imports: HashMap<String, ImportMode>,
}

impl Parse for BindgenConfig {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut path = None::<String>;
        let mut world = None::<String>;
        let mut store_data = None::<String>;
        let mut concurrent = false;
        let mut default_import = ImportMode::default();
        let mut imports = HashMap::new();

        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "path" => {
                    path = Some(content.parse::<LitStr>()?.value());
                }
                "world" => {
                    world = Some(content.parse::<LitStr>()?.value());
                }
                "store_data" => {
                    let ty: syn::Path = content.parse()?;
                    store_data = Some(quote_path(&ty));
                }
                "concurrent" => {
                    concurrent = content.parse::<LitBool>()?.value();
                }
                "imports" => {
                    let inner;
                    braced!(inner in content);
                    while !inner.is_empty() {
                        if inner.peek(LitStr) {
                            let name: LitStr = inner.parse()?;
                            inner.parse::<Token![:]>()?;
                            let mode = parse_import_mode(&inner)?;
                            imports.insert(name.value(), mode);
                        } else {
                            let kw: Ident = inner.parse()?;
                            if kw != "default" {
                                return Err(syn::Error::new(
                                    kw.span(),
                                    format!("expected string literal or `default`, found `{kw}`"),
                                ));
                            }
                            inner.parse::<Token![:]>()?;
                            default_import = parse_import_mode(&inner)?;
                        }
                        if inner.peek(Token![,]) {
                            inner.parse::<Token![,]>()?;
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown bindgen option `{other}`"),
                    ));
                }
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            path: path.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `path` field")
            })?,
            world: world.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `world` field")
            })?,
            store_data: store_data.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `store_data` field")
            })?,
            concurrent,
            default_import,
            imports,
        })
    }
}

fn parse_import_mode(input: ParseStream<'_>) -> Result<ImportMode> {
    let mut mode = ImportMode::default();
    loop {
        if input.peek(Token![async]) {
            input.parse::<Token![async]>()?;
            mode.is_async = true;
        } else {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "store" => mode.with_store = true,
                "trappable" => mode.trappable = true,
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown import mode flag `{other}`"),
                    ));
                }
            }
        }
        if input.peek(Token![|]) {
            input.parse::<Token![|]>()?;
        } else {
            break;
        }
    }
    Ok(mode)
}

fn quote_path(path: &syn::Path) -> String {
    let mut out = String::new();
    if path.leading_colon.is_some() {
        out.push_str("::");
    }
    for (i, seg) in path.segments.iter().enumerate() {
        if i > 0 {
            out.push_str("::");
        }
        let _ = write!(out, "{}", seg.ident);
    }
    out
}
