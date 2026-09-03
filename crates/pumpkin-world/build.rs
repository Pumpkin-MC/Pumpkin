#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

const ATTRS: &str = "\
#[allow(clippy::too_many_lines)]
#[allow(clippy::match_same_arms)]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::match_single_binding)]
#[must_use]
";

/// Base packs, relative to `CARGO_MANIFEST_DIR`.
///
/// These are applied first. Each pack must contain `data/<namespace>/`.
///
/// The first field is the logical datapack id exposed at runtime.
/// The second field is the path relative to `CARGO_MANIFEST_DIR`.
const DATAPACK_PACKS: &[(&str, &str)] = &[("vanilla", "../../assets/datapacks/26_2")];

/// Container dirs, relative to `CARGO_MANIFEST_DIR`.
///
/// Every immediate subdirectory is treated as a separate embedded datapack.
/// Container packs are applied after [`DATAPACK_PACKS`] in sorted order, so
/// later packs override earlier resources with the same resource id.
const DATAPACK_CONTAINERS: &[&str] = &["../../assets/tests/datapacks"];

/// Vanilla's implicit namespace: a bare `foo` is exactly `minecraft:foo`.
const DEFAULT_NAMESPACE: &str = "minecraft";

/// One registry's match arms, keyed by fully-qualified resource id.
///
/// Insertion overwrites, so a later datapack replaces an earlier datapack's
/// resource with the same id, matching vanilla datapack override semantics.
type Registry = BTreeMap<String, String>;

#[derive(Debug)]
struct EmbeddedPack {
    /// Logical datapack id exposed to the runtime.
    id: String,

    /// Physical source directory used at compile time.
    path: PathBuf,
}

#[derive(Default)]
struct Arms {
    templates: Registry,
    pools: Registry,
    template_pool_json: Registry,
    processor_list_json: Registry,
    test_instance_json: Registry,
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);

    // Load order:
    //
    // 1. Base packs.
    // 2. Container packs, sorted by directory name.
    //
    // Resources inserted later replace resources inserted earlier.
    let mut packs = Vec::<EmbeddedPack>::new();

    for &(id, relative_path) in DATAPACK_PACKS {
        println!("cargo:rerun-if-changed={relative_path}");

        packs.push(EmbeddedPack {
            id: id.to_string(),
            path: manifest_dir.join(relative_path),
        });
    }

    for container in DATAPACK_CONTAINERS {
        println!("cargo:rerun-if-changed={container}");

        let container_dir = manifest_dir.join(container);

        if !container_dir.is_dir() {
            println!(
                "cargo:warning=missing datapack container: {}",
                container_dir.display()
            );
            continue;
        }

        let mut found = fs::read_dir(&container_dir)
            .unwrap()
            .map(Result::unwrap)
            .filter(|entry| entry.path().is_dir())
            .collect::<Vec<_>>();

        // Deterministic datapack load order.
        found.sort_by_key(std::fs::DirEntry::file_name);

        for entry in found {
            let id = entry.file_name().into_string().unwrap();
            let path = entry.path();

            packs.push(EmbeddedPack { id, path });
        }
    }

    let mut arms = Arms::default();
    let mut embedded_pack_names = Vec::<String>::new();

    for pack in &packs {
        if embed_pack(&pack.path, &mut arms) {
            // A duplicate pack id can technically occur if multiple configured
            // locations contain a pack with the same name. Resources still use
            // normal load-order semantics, but the public pack list should only
            // contain the id once.
            if !embedded_pack_names.iter().any(|id| id == &pack.id) {
                embedded_pack_names.push(pack.id.clone());
            }
        }
    }

    let mut template_out = String::new();

    // ---------------------------------------------------------------------
    // Structure/template resource lookup functions
    // ---------------------------------------------------------------------

    template_out.push_str(&wrap_fn(
        "get_template_bytes",
        "path",
        "Option<&'static [u8]>",
        &arms.templates,
    ));

    template_out.push_str(&wrap_fn(
        "get_pool_elements",
        "pool_id",
        "Option<&'static [&'static str]>",
        &arms.pools,
    ));

    template_out.push_str(&wrap_fn(
        "get_template_pool_json",
        "path",
        "Option<&'static str>",
        &arms.template_pool_json,
    ));

    template_out.push_str(&wrap_fn(
        "get_processor_list_json",
        "path",
        "Option<&'static str>",
        &arms.processor_list_json,
    ));

    // ---------------------------------------------------------------------
    // Structure/template resource-name lists
    // ---------------------------------------------------------------------

    template_out.push_str(&name_list_fn(
        "_generated_all_template_names",
        &arms.templates,
    ));

    template_out.push_str(&name_list_fn("_generated_all_pool_names", &arms.pools));

    // ---------------------------------------------------------------------
    // Datapack identity
    // ---------------------------------------------------------------------

    template_out.push_str(&string_list_fn(
        "_generated_all_embedded_datapack_names",
        &embedded_pack_names,
    ));

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    fs::write(out_dir.join("template_embeddings.rs"), template_out).unwrap();

    // Test instances are a datapack registry of their own. Keep their generated
    // resource table separate from the structure-template cache so consumers do
    // not need to reach through `generation::structure::template`.
    let mut test_instance_out = String::new();
    test_instance_out.push_str(&wrap_fn(
        "get_test_instance_json",
        "path",
        "Option<&'static str>",
        &arms.test_instance_json,
    ));
    test_instance_out.push_str(&name_list_fn(
        "_generated_all_test_instance_names",
        &arms.test_instance_json,
    ));

    fs::write(
        out_dir.join("test_instance_embeddings.rs"),
        test_instance_out,
    )
    .unwrap();
}

/// Scans every namespace under `<pack>/data/`.
///
/// Returns `true` when the directory represents a valid datapack data root.
/// A valid but otherwise empty `data/` directory still counts as an embedded
/// datapack so it can be exposed through `/datapack list`.
fn embed_pack(pack_dir: &Path, arms: &mut Arms) -> bool {
    let data_dir = pack_dir.join("data");

    if !data_dir.is_dir() {
        println!(
            "cargo:warning=no data/ directory in pack: {}",
            pack_dir.display()
        );
        return false;
    }

    let mut namespaces = fs::read_dir(&data_dir)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| (entry.file_name().into_string().unwrap(), entry.path()))
        .collect::<Vec<_>>();

    namespaces.sort_by(|left, right| left.0.cmp(&right.0));

    for (namespace, namespace_dir) in namespaces {
        embed_namespace(&namespace, &namespace_dir, arms);
    }

    true
}

/// Emits embedded resources for all supported registries within a single
/// `data/<namespace>/` directory.
fn embed_namespace(namespace: &str, namespace_dir: &Path, arms: &mut Arms) {
    let structures_dir = namespace_dir.join("structure");

    if structures_dir.is_dir() {
        let mut pools: BTreeMap<String, Vec<String>> = BTreeMap::new();

        process_structure_dir(
            &structures_dir,
            "",
            &mut arms.templates,
            &mut pools,
            namespace,
        );

        for (pool_id, elements) in pools {
            let mut arm = format!("{} => Some(&[\n", patterns(namespace, &pool_id));

            for element in elements {
                let _ = writeln!(arm, "            \"{element}\",");
            }

            arm.push_str("        ]),");

            arms.pools.insert(qualify(namespace, &pool_id), arm);
        }
    }

    let worldgen_dir = namespace_dir.join("worldgen");

    process_json_dir(
        &worldgen_dir.join("template_pool"),
        "",
        &mut arms.template_pool_json,
        namespace,
    );

    process_json_dir(
        &worldgen_dir.join("processor_list"),
        "",
        &mut arms.processor_list_json,
        namespace,
    );

    process_json_dir(
        &namespace_dir.join("test_instance"),
        "",
        &mut arms.test_instance_json,
        namespace,
    );
}

/// Recursively emits `include_bytes!` match arms for every `.nbt` structure.
///
/// It also groups each directory's direct NBT children into the existing
/// structure-pool helper used elsewhere by Pumpkin.
fn process_structure_dir(
    dir: &Path,
    prefix: &str,
    registry: &mut Registry,
    pools: &mut BTreeMap<String, Vec<String>>,
    namespace: &str,
) {
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

            process_structure_dir(&path, &new_prefix, registry, pools, namespace);

            continue;
        }

        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("nbt"))
        {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy();

        let template_name = if prefix.is_empty() {
            stem.to_string()
        } else {
            format!("{prefix}/{stem}")
        };

        let resource_id = qualify(namespace, &template_name);
        let absolute_path = path.canonicalize().unwrap();

        registry.insert(
            resource_id.clone(),
            format!(
                "{patterns} => Some(include_bytes!(r#\"{path}\"#)),",
                patterns = patterns(namespace, &template_name),
                path = absolute_path.display(),
            ),
        );

        if !prefix.is_empty() {
            pools
                .entry(prefix.to_string())
                .or_default()
                .push(resource_id);
        }
    }
}

/// Recursively emits `include_str!` match arms for every `.json` resource.
fn process_json_dir(dir: &Path, prefix: &str, registry: &mut Registry, namespace: &str) {
    if !dir.is_dir() {
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

            process_json_dir(&path, &new_prefix, registry, namespace);

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

        let absolute_path = path.canonicalize().unwrap();

        registry.insert(
            qualify(namespace, &id),
            format!(
                "{patterns} => Some(include_str!(r#\"{path}\"#)),",
                patterns = patterns(namespace, &id),
                path = absolute_path.display(),
            ),
        );
    }
}

/// Returns a fully-qualified Minecraft resource id.
fn qualify(namespace: &str, id: &str) -> String {
    format!("{namespace}:{id}")
}

/// Produces match patterns for an embedded resource.
///
/// Resources in the default namespace additionally accept their bare id,
/// because vanilla treats `foo` and `minecraft:foo` as equivalent.
fn patterns(namespace: &str, id: &str) -> String {
    if namespace == DEFAULT_NAMESPACE {
        format!("        \"{namespace}:{id}\" | \"{id}\"")
    } else {
        format!("        \"{namespace}:{id}\"")
    }
}

/// Generates one resource lookup function.
fn wrap_fn(name: &str, argument: &str, return_type: &str, registry: &Registry) -> String {
    let mut arms = String::new();

    for arm in registry.values() {
        let _ = writeln!(arms, "{arm}");
    }

    format!(
        "{ATTRS}\
         pub fn {name}({argument}: &str) -> {return_type} {{\n\
         \x20   match {argument} {{\n\
         {arms}\
         \x20       _ => None,\n\
         \x20   }}\n\
         }}\n\n"
    )
}

/// Generates a list containing every resource id in a registry.
fn name_list_fn(name: &str, registry: &Registry) -> String {
    let mut output = format!(
        "#[must_use]\n\
         #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]\n\
         pub const fn {name}() -> &'static [&'static str] {{\n\
         \x20   &[\n"
    );

    for id in registry.keys() {
        let _ = writeln!(output, "        \"{id}\",");
    }

    output.push_str("    ]\n}\n\n");

    output
}

/// Generates a static list of strings.
///
/// Unlike [`name_list_fn`], this is used for values that are not resource ids,
/// such as logical embedded datapack names.
fn string_list_fn(name: &str, values: &[String]) -> String {
    let mut output = format!(
        "#[must_use]\n\
         #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]\n\
         pub const fn {name}() -> &'static [&'static str] {{\n\
         \x20   &[\n"
    );

    for value in values {
        // Debug formatting produces a properly escaped Rust string literal.
        let _ = writeln!(output, "        {value:?},");
    }

    output.push_str("    ]\n}\n\n");

    output
}
