use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("template_embeddings.rs");

    let mut code = String::from(
        "pub fn get_template_bytes(path: &str) -> Option<&'static [u8]> {\n    match path {\n",
    );
    let mut pool_code = String::from(
        "pub fn get_pool_elements(pool_id: &str) -> Option<&'static [&'static str]> {\n    match pool_id {\n",
    );

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets_dir = Path::new(&manifest_dir).join("assets/structures");
    if assets_dir.exists() {
        let mut pools = std::collections::BTreeMap::new();
        process_dir(&assets_dir, "", &mut code, &mut pools);

        for (pool_id, elements) in pools {
            pool_code.push_str(&format!(
                "        \"minecraft:{}\" | \"{}\" => Some(&[\n",
                pool_id, pool_id
            ));
            for element in elements {
                pool_code.push_str(&format!("            \"{}\",\n", element));
            }
            pool_code.push_str("        ]),\n");
        }
    }

    code.push_str("        _ => None,\n");
    code.push_str("    }\n}\n");

    pool_code.push_str("        _ => None,\n");
    pool_code.push_str("    }\n}\n");

    fs::write(&dest_path, format!("{}\n{}", code, pool_code)).unwrap();
    println!("cargo:rerun-if-changed=assets/structures");
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
                format!("{}/{}", prefix, name)
            };
            process_dir(&path, &new_prefix, code, pools);
        } else if name.ends_with(".nbt") {
            let template_name = format!("{}/{}", prefix, name.strip_suffix(".nbt").unwrap());
            let abs_path = path.canonicalize().unwrap();
            code.push_str(&format!(
                "        \"{}\" => Some(include_bytes!(r#\"{}\"#)),\n",
                template_name,
                abs_path.display()
            ));

            if !prefix.is_empty() {
                pools
                    .entry(prefix.to_string())
                    .or_default()
                    .push(template_name);
            }
        }
    }
}
