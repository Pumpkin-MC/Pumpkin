use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use proc_macro2::TokenStream;
use quote::quote;

const DEFAULT_NAMESPACE: &str = "minecraft";

pub fn build() -> TokenStream {
    let int_dir = Path::new("../../assets/datapack/data/minecraft/context_int_provider");
    let float_dir = Path::new("../../assets/datapack/data/minecraft/context_float_provider");

    let mut int_providers: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut float_providers: BTreeMap<String, (String, String)> = BTreeMap::new();

    if int_dir.is_dir() {
        scan_dir(int_dir, "", &mut int_providers);
    }

    if float_dir.is_dir() {
        scan_dir(float_dir, "", &mut float_providers);
    }

    let mut int_arms = Vec::new();
    for (qualified, (bare, rel_path)) in &int_providers {
        int_arms.push(quote! {
            #qualified | #bare => Some(include_str!(#rel_path)),
        });
    }

    let mut float_arms = Vec::new();
    for (qualified, (bare, rel_path)) in &float_providers {
        float_arms.push(quote! {
            #qualified | #bare => Some(include_str!(#rel_path)),
        });
    }

    let int_names: Vec<&str> = int_providers.keys().map(String::as_str).collect();
    let float_names: Vec<&str> = float_providers.keys().map(String::as_str).collect();

    quote! {
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[allow(clippy::missing_const_for_fn)]
        #[allow(clippy::match_single_binding)]
        #[must_use]
        pub fn get_context_int_provider_json(id: &str) -> Option<&'static str> {
            match id {
                #( #int_arms )*
                _ => None,
            }
        }

        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[allow(clippy::missing_const_for_fn)]
        #[allow(clippy::match_single_binding)]
        #[must_use]
        pub fn get_context_float_provider_json(id: &str) -> Option<&'static str> {
            match id {
                #( #float_arms )*
                _ => None,
            }
        }

        #[must_use]
        #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
        pub const fn all_context_int_provider_names() -> &'static [&'static str] {
            &[
                #( #int_names ),*
            ]
        }

        #[must_use]
        #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
        pub const fn all_context_float_provider_names() -> &'static [&'static str] {
            &[
                #( #float_names ),*
            ]
        }
    }
}

fn scan_dir(dir: &Path, prefix: &str, registry: &mut BTreeMap<String, (String, String)>) {
    let mut entries = fs::read_dir(dir)
        .expect("read dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            let new_prefix = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            scan_dir(&path, &new_prefix, registry);
            continue;
        }

        let Some(stem) = name.strip_suffix(".json") else {
            continue;
        };

        let id = if prefix.is_empty() {
            stem.to_string()
        } else {
            format!("{prefix}/{stem}")
        };

        let qualified = format!("{DEFAULT_NAMESPACE}:{id}");
        let bare = id;

        // Path relative to crates/pumpkin-data/src/generated/
        let rel_to_repo = path.strip_prefix("../../").expect("strip ../../");
        let rel_path = format!(
            "../../../../{}",
            rel_to_repo.to_string_lossy().replace('\\', "/")
        );

        registry.insert(qualified, (bare, rel_path));
    }
}
