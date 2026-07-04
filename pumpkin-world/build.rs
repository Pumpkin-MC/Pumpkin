use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("template_embeddings.rs");

    let mut code = String::from(
        "
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[must_use]
        pub fn get_template_bytes(path: &str) -> Option<&'static [u8]> {\n    match path {\n",
    );
    let mut pool_code = String::from(
        "
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[must_use]
        pub fn get_pool_elements(pool_id: &str) -> Option<&'static [&'static str]> {\n    match pool_id {\n",
    );
    let mut template_pool_json_code = String::from(
        "
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[must_use]
        pub fn get_template_pool_json(path: &str) -> Option<&'static str> {\n    match path {\n",
    );
    let mut processor_list_json_code = String::from(
        "
        #[allow(clippy::too_many_lines)]
        #[allow(clippy::match_same_arms)]
        #[must_use]
        pub fn get_processor_list_json(path: &str) -> Option<&'static str> {\n    match path {\n",
    );

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets_dir = Path::new(&manifest_dir).join("assets/structures");
    if assets_dir.exists() {
        let mut pools = std::collections::BTreeMap::new();
        process_dir(&assets_dir, "", &mut code, &mut pools);

        for (pool_id, elements) in pools {
            let _ = writeln!(
                pool_code,
                "        \"minecraft:{pool_id}\" | \"{pool_id}\" => Some(&["
            );
            for element in elements {
                let _ = writeln!(pool_code, "            \"{element}\",");
            }
            pool_code.push_str("        ]),\n");
        }
    }

    code.push_str("        _ => None,\n");
    code.push_str("    }\n}\n");

    pool_code.push_str("        _ => None,\n");
    pool_code.push_str("    }\n}\n");

    let worldgen_dir = Path::new(&manifest_dir).join("assets/worldgen");
    process_json_dir(
        &worldgen_dir.join("template_pool"),
        "",
        &mut template_pool_json_code,
    );
    process_json_dir(
        &worldgen_dir.join("processor_list"),
        "",
        &mut processor_list_json_code,
    );
    template_pool_json_code.push_str("        _ => None,\n");
    template_pool_json_code.push_str("    }\n}\n");
    processor_list_json_code.push_str("        _ => None,\n");
    processor_list_json_code.push_str("    }\n}\n");

    fs::write(
        &dest_path,
        format!("{code}\n{pool_code}\n{template_pool_json_code}\n{processor_list_json_code}"),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=assets/structures");
    println!("cargo:rerun-if-changed=assets/worldgen");
}

fn process_dir(
    dir: &Path,
    prefix: &str,
    code: &mut String,
    pools: &mut std::collections::BTreeMap<String, Vec<String>>,
) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();

        if path.is_dir() {
            let new_prefix = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            process_dir(&path, &new_prefix, code, pools);
        } else if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nbt"))
        {
            let stem = path.file_stem().unwrap().to_string_lossy();
            let template_name = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}/{stem}")
            };
            let abs_path = path.canonicalize().unwrap();
            let _ = writeln!(
                code,
                "        \"{template_name}\" => Some(include_bytes!(r#\"{abs}\"#)),",
                template_name = template_name,
                abs = abs_path.display()
            );

            if !prefix.is_empty() {
                pools
                    .entry(prefix.to_string())
                    .or_default()
                    .push(template_name);
            }
        }
    }
}

fn process_json_dir(dir: &Path, prefix: &str, code: &mut String) {
    if !dir.exists() {
        return;
    }

    let mut entries = fs::read_dir(dir)
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();
        if path.is_dir() {
            let new_prefix = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            process_json_dir(&path, &new_prefix, code);
        } else if let Some(stem) = name.strip_suffix(".json") {
            let id = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}/{stem}")
            };
            let abs_path = path.canonicalize().unwrap();
            let _ = writeln!(
                code,
                "        \"minecraft:{id}\" | \"{id}\" => Some(include_str!(r#\"{abs}\"#)),",
                id = id,
                abs = abs_path.display()
            );
        }
    }
}
