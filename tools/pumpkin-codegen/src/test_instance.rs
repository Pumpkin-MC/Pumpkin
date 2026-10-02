use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use quote::quote;

const DEFAULT_NAMESPACE: &str = "minecraft";

struct EmbeddedPack {
    id: String,
    path: PathBuf,
}

pub fn build() -> TokenStream {
    let base_packs = [("vanilla", Path::new("../../assets/datapack"))];
    let container_dir = Path::new("../../assets/tests/datapacks");

    let mut packs = Vec::new();
    for (id, path) in base_packs {
        packs.push(EmbeddedPack {
            id: id.to_string(),
            path: path.to_path_buf(),
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
            let id = entry.file_name().to_string_lossy().to_string();
            packs.push(EmbeddedPack {
                id,
                path: entry.path(),
            });
        }
    }

    let mut test_instances: BTreeMap<String, (bool, String, String)> = BTreeMap::new();

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
            let ti_dir = ns_dir.join("test_instance");
            if ti_dir.is_dir() {
                scan_json_dir(&ti_dir, "", &namespace, &mut test_instances);
            }
        }
    }

    let mut arms = Vec::new();
    for (resource_id, (is_default, bare_id, rel_path)) in &test_instances {
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

    let names: Vec<&str> = test_instances.keys().map(String::as_str).collect();

    quote! {
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[allow(clippy::missing_const_for_fn)]
        #[allow(clippy::match_single_binding)]
        #[must_use]
        pub fn get_test_instance_json(path: &str) -> Option<&'static str> {
            match path {
                #( #arms )*
                _ => None,
            }
        }

        #[must_use]
        #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
        pub const fn all_test_instance_names() -> &'static [&'static str] {
            &[
                #( #names ),*
            ]
        }
    }
}

fn scan_json_dir(
    dir: &Path,
    prefix: &str,
    namespace: &str,
    test_instances: &mut BTreeMap<String, (bool, String, String)>,
) {
    let mut entries = fs::read_dir(dir)
        .expect("read json dir")
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
            scan_json_dir(&path, &new_prefix, namespace, test_instances);
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

        let resource_id = format!("{namespace}:{id}");
        let is_default = namespace == DEFAULT_NAMESPACE;
        let bare_id = id;

        // Path relative to crates/pumpkin-data/src/generated/
        let rel_to_repo = path.strip_prefix("../../").expect("strip ../../");
        let rel_path = format!(
            "../../../../{}",
            rel_to_repo.to_string_lossy().replace('\\', "/")
        );

        test_instances.insert(resource_id, (is_default, bare_id, rel_path));
    }
}
