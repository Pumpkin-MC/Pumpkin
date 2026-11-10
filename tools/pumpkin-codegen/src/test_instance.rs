use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

const DEFAULT_NAMESPACE: &str = "minecraft";

struct EmbeddedPack {
    path: PathBuf,
}

pub fn build() -> TokenStream {
    build_resource(
        "test_instance",
        "json",
        true,
        "get_test_instance_json",
        "all_test_instance_names",
    )
}

pub fn build_environment() -> TokenStream {
    build_resource(
        "test_environment",
        "json",
        true,
        "get_test_environment_json",
        "all_test_environment_names",
    )
}

pub fn build_function() -> TokenStream {
    build_resource(
        "function",
        "mcfunction",
        false,
        "get_test_function_source",
        "all_test_function_names",
    )
}

fn build_resource(
    directory: &str,
    extension: &str,
    include_vanilla: bool,
    getter: &str,
    names_getter: &str,
) -> TokenStream {
    let container_dir = Path::new("../../assets/tests/datapacks");

    let mut packs = Vec::new();
    if include_vanilla {
        packs.push(EmbeddedPack {
            path: PathBuf::from("../../assets/datapack"),
        });
    }

    if container_dir.is_dir() {
        let mut entries = fs::read_dir(container_dir)
            .expect("read container dir")
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .collect::<Vec<_>>();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            packs.push(EmbeddedPack { path: entry.path() });
        }
    }

    let mut resources: BTreeMap<String, (bool, String, String)> = BTreeMap::new();

    for pack in &packs {
        let data_dir = pack.path.join("data");
        if !data_dir.is_dir() {
            continue;
        }

        let mut namespaces = fs::read_dir(&data_dir)
            .expect("read data dir")
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
            .collect::<Vec<_>>();
        namespaces.sort_by(|a, b| a.0.cmp(&b.0));

        for (namespace, ns_dir) in namespaces {
            let resource_dir = ns_dir.join(directory);
            if resource_dir.is_dir() {
                scan_resource_dir(&resource_dir, "", &namespace, extension, &mut resources);
            }
        }
    }

    let mut arms = Vec::new();
    for (resource_id, (is_default, bare_id, rel_path)) in &resources {
        if *is_default {
            arms.push(quote! {
                #resource_id | #bare_id => Some(include_str!(#rel_path)),
            });
        } else {
            arms.push(quote! {
                #resource_id => Some(include_str!(#rel_path)),
            });
        }
    }

    let names: Vec<&str> = resources.keys().map(String::as_str).collect();
    let getter = format_ident!("{getter}");
    let names_getter = format_ident!("{names_getter}");

    quote! {
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[allow(clippy::missing_const_for_fn)]
        #[allow(clippy::match_single_binding)]
        #[must_use]
        pub fn #getter(path: &str) -> Option<&'static str> {
            match path {
                #( #arms )*
                _ => None,
            }
        }

        #[must_use]
        #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
        pub const fn #names_getter() -> &'static [&'static str] {
            &[
                #( #names ),*
            ]
        }
    }
}

fn scan_resource_dir(
    dir: &Path,
    prefix: &str,
    namespace: &str,
    extension: &str,
    resources: &mut BTreeMap<String, (bool, String, String)>,
) {
    let mut entries = fs::read_dir(dir)
        .expect("read resource dir")
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
            scan_resource_dir(&path, &new_prefix, namespace, extension, resources);
            continue;
        }

        let Some(stem) = name.strip_suffix(&format!(".{extension}")) else {
            continue;
        };

        let id = if prefix.is_empty() {
            stem.to_string()
        } else {
            format!("{prefix}/{stem}")
        };

        let resource_id = format!("{namespace}:{id}");
        let is_default = namespace == DEFAULT_NAMESPACE;
        let bare_id = id;

        // Path relative to crates/pumpkin-data/src/generated/
        let rel_to_repo = path.strip_prefix("../../").expect("strip ../../");
        let rel_path = format!(
            "../../../../{}",
            rel_to_repo.to_string_lossy().replace('\\', "/")
        );

        resources.insert(resource_id, (is_default, bare_id, rel_path));
    }
}
