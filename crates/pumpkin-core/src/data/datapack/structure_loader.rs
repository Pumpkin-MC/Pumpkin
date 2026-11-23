use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::error;

use rayon::prelude::*;

use pumpkin_world::generation::structure::{
    structures::jigsaw::{
        PreparedTemplatePool, parse_dynamic_template_pool, register_dynamic_template_pool,
    },
    template::processor::{
        StructureProcessor, parse_dynamic_processor_list, register_prepared_processor_list,
    },
};

#[derive(Default)]
pub(super) struct PreparedStructures {
    pub(super) templates: Vec<(String, Arc<[u8]>)>,
    pub(super) pools: Vec<(String, PreparedTemplatePool)>,
    pub(super) processors: Vec<(String, Arc<[StructureProcessor]>)>,
}

impl PreparedStructures {
    pub(super) fn apply(self, reload: Option<&std::sync::RwLockWriteGuard<'_, ()>>) {
        for (id, bytes) in self.templates {
            pumpkin_world::generation::structure::template::global_cache()
                .register_template(&id, bytes);
        }
        let pools: Vec<_> = self
            .pools
            .into_par_iter()
            .map(|(id, pool)| {
                let pool = match reload {
                    Some(reload) => pool.resolve_with(|name| {
                        pumpkin_world::generation::structure::template::global_cache()
                            .get_during_reload(name, reload)
                    }),
                    None => pool.resolve(),
                };
                (id, pool)
            })
            .collect();
        for (id, pool) in pools {
            register_dynamic_template_pool(&id, pool);
        }
        for (id, processors) in self.processors {
            register_prepared_processor_list(&id, processors);
        }
    }
}

use super::file_loader::{collect_files, resource_path_without_extension};

/// Recursively scans `<namespace_dir>/structure/` for `.nbt` structure templates
/// and registers them into [`pumpkin_world::generation::structure::template::global_cache()`].
#[must_use]
pub fn load_structures_from_dir(namespace: &str, structure_dir: &Path) -> usize {
    let prepared = prepare_structures_from_dir(namespace, structure_dir);
    let count = prepared.templates.len();
    prepared.apply(None);
    count
}

pub(super) fn prepare_structures_from_dir(
    namespace: &str,
    structure_dir: &Path,
) -> PreparedStructures {
    let files = collect_files(structure_dir, "nbt");
    let templates = files
        .par_iter()
        .filter_map(|path| {
            let template_name = resource_path_without_extension(structure_dir, path)?;
            let resource_id = format!("{namespace}:{template_name}");
            match fs::read(path) {
                Ok(bytes) => Some((resource_id, Arc::from(bytes.into_boxed_slice()))),
                Err(e) => {
                    error!("Failed to read structure template '{resource_id}' from {path:?}: {e}");
                    None
                }
            }
        })
        .collect();
    PreparedStructures {
        templates,
        ..PreparedStructures::default()
    }
}

/// Recursively scans `<namespace_dir>/worldgen/template_pool/` for `.json` template pool definitions.
///
/// Registers them into [`pumpkin_world::generation::structure::structures::jigsaw::register_dynamic_template_pool_json`].
#[must_use]
pub fn load_template_pools_from_dir(namespace: &str, template_pool_dir: &Path) -> usize {
    let prepared = prepare_template_pools_from_dir(namespace, template_pool_dir);
    let count = prepared.pools.len();
    prepared.apply(None);
    count
}

pub(super) fn prepare_template_pools_from_dir(
    namespace: &str,
    template_pool_dir: &Path,
) -> PreparedStructures {
    let files = collect_files(template_pool_dir, "json");
    let pools = files
        .par_iter()
        .filter_map(|path| {
            let pool_name = resource_path_without_extension(template_pool_dir, path)?;
            let resource_id = format!("{namespace}:{pool_name}");
            match fs::read_to_string(path) {
                Ok(json_content) => {
                    match parse_dynamic_template_pool(&resource_id, &json_content) {
                        Ok(pool) => Some((resource_id, pool)),
                        Err(e) => {
                            error!(
                                "Failed to parse template pool '{resource_id}' from {path:?}: {e}"
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read template pool JSON '{resource_id}' from {path:?}: {e}");
                    None
                }
            }
        })
        .collect();
    PreparedStructures {
        pools,
        ..PreparedStructures::default()
    }
}

/// Recursively scans `<namespace_dir>/worldgen/processor_list/` for `.json` processor lists.
///
/// Registers them into [`pumpkin_world::generation::structure::template::processor::register_dynamic_processor_list`].
#[must_use]
pub fn load_processor_lists_from_dir(namespace: &str, processor_list_dir: &Path) -> usize {
    let prepared = prepare_processor_lists_from_dir(namespace, processor_list_dir);
    let count = prepared.processors.len();
    prepared.apply(None);
    count
}

pub(super) fn prepare_processor_lists_from_dir(
    namespace: &str,
    processor_list_dir: &Path,
) -> PreparedStructures {
    let files = collect_files(processor_list_dir, "json");
    let processors = files
        .par_iter()
        .filter_map(|path| {
            let list_name = resource_path_without_extension(processor_list_dir, path)?;
            let resource_id = format!("{namespace}:{list_name}");
            match fs::read_to_string(path) {
                Ok(json_content) => match parse_dynamic_processor_list(&json_content) {
                    Ok(processors) => Some((resource_id, processors)),
                    Err(e) => {
                        error!("Failed to parse processor list '{resource_id}' from {path:?}: {e}");
                        None
                    }
                },
                Err(e) => {
                    error!("Failed to read processor list JSON '{resource_id}' from {path:?}: {e}");
                    None
                }
            }
        })
        .collect();
    PreparedStructures {
        processors,
        ..PreparedStructures::default()
    }
}

/// Clears all dynamically registered worldgen structures, template pools, and processor lists.
pub fn clear_dynamic_worldgen_data() {
    pumpkin_world::generation::structure::template::global_cache().clear_dynamic();
    pumpkin_world::generation::structure::structures::jigsaw::clear_dynamic_template_pools();
    pumpkin_world::generation::structure::template::processor::clear_dynamic_processors();
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_world::generation::structure::{DYNAMIC_WORLDGEN_RELOAD, template::get_template};
    use std::{sync::mpsc, time::Duration};
    use tempfile::tempdir;

    #[test]
    fn load_structure_template_and_template_pool() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("data").join("test_ns");
        let structure_dir = data_dir.join("structure");
        let template_pool_dir = data_dir.join("worldgen").join("template_pool");
        fs::create_dir_all(&structure_dir).unwrap();
        fs::create_dir_all(&template_pool_dir).unwrap();

        // Write a test template pool JSON
        let pool_json = r#"{
            "fallback": "minecraft:empty",
            "elements": [
                {
                    "weight": 10,
                    "element": {
                        "element_type": "minecraft:empty_pool_element"
                    }
                }
            ]
        }"#;
        fs::write(template_pool_dir.join("test_pool.json"), pool_json).unwrap();

        let pools_loaded = load_template_pools_from_dir("test_ns", &template_pool_dir);
        assert_eq!(pools_loaded, 1);

        let discovered =
            pumpkin_world::generation::structure::structures::jigsaw::TemplatePool::discover(
                "test_ns:test_pool",
            );
        assert!(discovered.is_some());
        let pool = discovered.unwrap();
        assert_eq!(pool.elements.len(), 1);
        assert_eq!(pool.elements[0].weight, 10);

        fs::write(
            template_pool_dir.join("test_pool.json"),
            pool_json.replace("10", "20"),
        )
        .unwrap();
        let prepared = prepare_template_pools_from_dir("test_ns", &template_pool_dir);
        let current =
            pumpkin_world::generation::structure::structures::jigsaw::TemplatePool::discover(
                "test_ns:test_pool",
            )
            .unwrap();
        assert_eq!(current.elements[0].weight, 10);
        prepared.apply(None);
        let replaced =
            pumpkin_world::generation::structure::structures::jigsaw::TemplatePool::discover(
                "test_ns:test_pool",
            )
            .unwrap();
        assert_eq!(replaced.elements[0].weight, 20);

        // Pool preparation must not retain a template from the preceding pack.
        let cache = pumpkin_world::generation::structure::template::global_cache();
        let make_template = |size| {
            let mut nbt = pumpkin_nbt::NbtCompound::new();
            nbt.put(
                "size",
                pumpkin_nbt::tag::NbtTag::List(vec![pumpkin_nbt::tag::NbtTag::Int(size); 3]),
            );
            nbt.put("blocks", pumpkin_nbt::tag::NbtTag::List(Vec::new()));
            nbt.put("palette", pumpkin_nbt::tag::NbtTag::List(Vec::new()));
            Arc::from(pumpkin_nbt::nbt_compress::write_gzip_compound_tag_to_bytes(nbt).unwrap())
        };
        cache.register_template("test_ns:deferred", make_template(1));
        let pool_json = r#"{
            "fallback":"minecraft:empty",
            "elements":[{"weight":1,"element":{
                "element_type":"minecraft:single_pool_element",
                "location":"test_ns:deferred","processors":"minecraft:empty","projection":"rigid"
            }}]
        }"#;
        fs::write(template_pool_dir.join("deferred.json"), pool_json).unwrap();
        let prepared = prepare_template_pools_from_dir("test_ns", &template_pool_dir);
        let reload = DYNAMIC_WORLDGEN_RELOAD.write().unwrap();
        cache.remove_dynamic("test_ns:deferred");
        let (tx, rx) = mpsc::channel();
        let (started_tx, started_rx) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            tx.send(get_template("test_ns:deferred")).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(100)).is_err());
        cache.register_template("test_ns:deferred", make_template(2));
        prepared.apply(Some(&reload));
        drop(reload);
        let observed = rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
        assert_eq!(observed.size.x, 2);
        reader.join().unwrap();
        let pool =
            pumpkin_world::generation::structure::structures::jigsaw::TemplatePool::discover(
                "test_ns:deferred",
            )
            .unwrap();
        assert_eq!(pool.elements[0].first_template().unwrap().size.x, 2);

        clear_dynamic_worldgen_data();
    }
}
