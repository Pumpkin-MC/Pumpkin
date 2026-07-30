use std::{
    io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use arc_swap::ArcSwap;
use tokio::{fs::OpenOptions, io::AsyncReadExt, sync::Mutex};
use toml::{Table, map::Map};

use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::config::{
    ConfigPath, ConfigPathElement, ConfigTree, ConfigValue,
};

enum ConfigLoadState {
    Loaded {
        table: Arc<Mutex<toml::Table>>,
        changed: Arc<AtomicBool>,
    },
    /// The config file does not exist.
    DoesNotExist,
    /// It is unknown whether the config file exists. On the next
    /// get or set, this should be checked.
    Unknown,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigLoadError {
    #[error("IO Error: {0}")]
    Io(io::Error),
    #[error("Error parsing: {0}")]
    Deserialize(toml::de::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigSetError {
    #[error("{0}")]
    Load(ConfigLoadError),
    #[error("Invalid tree")]
    InvalidTree,
    #[error("Invalid path")]
    InvalidPath,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigSaveError {
    #[error("IO Error: {0}")]
    Io(io::Error),
    #[error("Error serializing: {0}")]
    Serialize(toml::ser::Error),
}

pub struct PluginConfigManager {
    config: ArcSwap<ConfigLoadState>,
    path: PathBuf,
}

impl PluginConfigManager {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            config: ArcSwap::new(Arc::new(ConfigLoadState::Unknown)),
            path,
        }
    }

    async fn get_or_load_config_if_exists(
        &self,
    ) -> Result<Option<Arc<Mutex<toml::Table>>>, ConfigLoadError> {
        let config_guard = self.config.load();
        if let ConfigLoadState::Loaded { table, changed: _ } = &**config_guard {
            return Ok(Some(Arc::clone(table)));
        }
        if let ConfigLoadState::DoesNotExist = &**config_guard {
            return Ok(None);
        };
        drop(config_guard);

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(ConfigLoadError::Io)?;
        }
        let mut file = match OpenOptions::new()
            .create(false)
            .read(true)
            .write(false)
            .open(&self.path)
            .await
        {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.config.store(Arc::new(ConfigLoadState::DoesNotExist));
                return Ok(None);
            }
            Err(e) => return Err(ConfigLoadError::Io(e)),
        };
        let mut string = String::new();
        file.read_to_string(&mut string)
            .await
            .map_err(ConfigLoadError::Io)?;
        let table = Arc::new(Mutex::new(
            string.parse().map_err(ConfigLoadError::Deserialize)?,
        ));
        self.config.store(Arc::new(ConfigLoadState::Loaded {
            table: Arc::clone(&table),
            changed: Arc::new(AtomicBool::new(false)),
        }));
        Ok(Some(table))
    }

    async fn get_or_load_config(
        &self,
    ) -> Result<(Arc<Mutex<toml::Table>>, Arc<AtomicBool>), ConfigLoadError> {
        if let ConfigLoadState::Loaded { table, changed } = &**self.config.load() {
            return Ok((Arc::clone(table), Arc::clone(changed)));
        }

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(ConfigLoadError::Io)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(false)
            .open(&self.path)
            .await
            .map_err(ConfigLoadError::Io)?;
        let mut string = String::new();
        file.read_to_string(&mut string)
            .await
            .map_err(ConfigLoadError::Io)?;
        let table = Arc::new(Mutex::new(
            string.parse().map_err(ConfigLoadError::Deserialize)?,
        ));
        let changed = Arc::new(AtomicBool::new(false));
        self.config.store(Arc::new(ConfigLoadState::Loaded {
            table: Arc::clone(&table),
            changed: Arc::clone(&changed),
        }));

        Ok((table, changed))
    }

    pub async fn get(&self, k: ConfigPath) -> Result<Option<ConfigTree>, ConfigLoadError> {
        let Some(table) = self.get_or_load_config_if_exists().await? else {
            return Ok(None);
        };
        let table = table.lock().await;
        Ok(match index_table(&table, k) {
            Some(either::Left(value)) => Some(value.clone().into()),
            Some(either::Right(table)) => Some(toml::Value::Table(table.clone()).into()),
            None => None,
        })
    }

    pub async fn set(&self, k: ConfigPath, v: ConfigTree) -> Result<(), ConfigSetError> {
        let Ok(value) = Option::<toml::Value>::try_from(v) else {
            return Err(ConfigSetError::InvalidTree);
        };
        let Some(value) = value else {
            self.remove(k).await?;
            return Ok(());
        };
        let (table, changed) = self
            .get_or_load_config()
            .await
            .map_err(ConfigSetError::Load)?;
        changed.store(true, Ordering::Release);
        let mut table = table.lock().await;

        let mut current_value: either::Either<&mut toml::Value, &mut Table> =
            either::Right(&mut table);
        let mut k_iter = k.into_iter().peekable();
        while let Some(elem) = k_iter.next() {
            match elem {
                ConfigPathElement::Key(name) => {
                    let table = match current_value {
                        either::Left(toml::Value::Table(table)) => table,
                        either::Right(table) => table,
                        _ => return Err(ConfigSetError::InvalidPath),
                    };

                    let insert_fn = match k_iter.peek() {
                        None => {
                            table.insert(name, value);
                            return Ok(());
                        }
                        Some(ConfigPathElement::Index(_)) => || toml::Value::Array(Vec::new()),
                        Some(ConfigPathElement::Key(_)) => || toml::Value::Table(Map::new()),
                    };

                    current_value = either::Left(table.entry(name).or_insert_with(insert_fn));
                }
                ConfigPathElement::Index(i) => {
                    let elem = if let either::Left(toml::Value::Array(arr)) = current_value {
                        if i as usize == arr.len() {
                            match k_iter.peek() {
                                None => {
                                    arr.push(value);
                                    return Ok(());
                                }
                                Some(ConfigPathElement::Index(_)) => {
                                    current_value =
                                        either::Left(arr.push_mut(toml::Value::Array(Vec::new())));
                                }
                                Some(ConfigPathElement::Key(_)) => {
                                    current_value =
                                        either::Left(arr.push_mut(toml::Value::Table(Map::new())));
                                }
                            }
                            continue;
                        }
                        arr.get_mut(i as usize)
                    } else {
                        return Err(ConfigSetError::InvalidPath);
                    };
                    let Some(elem) = elem else {
                        return Err(ConfigSetError::InvalidPath);
                    };
                    current_value = either::Left(elem);
                }
            }
        }

        Ok(())
    }

    pub async fn remove(&self, k: ConfigPath) -> Result<bool, ConfigSetError> {
        let (table, changed) = self
            .get_or_load_config()
            .await
            .map_err(ConfigSetError::Load)?;
        changed.store(true, Ordering::Release);
        let mut table = table.lock().await;
        let mut last_value: either::Either<&mut Vec<toml::Value>, &mut Map<String, toml::Value>> =
            either::Right(&mut table);
        let mut k_iter = k.into_iter().peekable();
        while let Some(elem) = k_iter.next() {
            if k_iter.peek().is_none() {
                return match (elem, last_value) {
                    (ConfigPathElement::Key(name), either::Right(table)) => {
                        Ok(table.remove(&name).is_some())
                    }
                    (ConfigPathElement::Index(i), either::Left(arr)) => {
                        if arr.len() <= i as usize {
                            Ok(false)
                        } else {
                            // Will not panic because we do a manual length check above.
                            // `Vec::try_remove` is nightly-only as of writing this.
                            arr.remove(i as usize);
                            Ok(true)
                        }
                    }
                    _ => Err(ConfigSetError::InvalidPath),
                };
            }
            last_value = match (elem, last_value) {
                (ConfigPathElement::Key(name), either::Right(table)) => {
                    match table.get_mut(&name) {
                        Some(toml::Value::Table(table)) => either::Right(table),
                        Some(toml::Value::Array(arr)) => either::Left(arr),
                        None => return Ok(false),
                        _ => return Err(ConfigSetError::InvalidPath),
                    }
                }
                (ConfigPathElement::Index(i), either::Left(arr)) => match arr.get_mut(i as usize) {
                    Some(toml::Value::Table(table)) => either::Right(table),
                    Some(toml::Value::Array(arr)) => either::Left(arr),
                    None => return Ok(false),
                    _ => return Err(ConfigSetError::InvalidPath),
                },
                _ => return Err(ConfigSetError::InvalidPath),
            };
        }
        unreachable!();
    }

    pub async fn has_key(&self, k: ConfigPath) -> Result<bool, ConfigLoadError> {
        let Some(table) = self.get_or_load_config_if_exists().await? else {
            return Ok(false);
        };

        let table = table.lock().await;
        let mut last_value: either::Either<&toml::Value, &Map<String, toml::Value>> =
            either::Right(&table);
        for elem in k {
            match (elem, last_value) {
                (ConfigPathElement::Key(name), either::Left(toml::Value::Table(table)))
                | (ConfigPathElement::Key(name), either::Right(table)) => {
                    let Some(value) = table.get(&name) else {
                        return Ok(false);
                    };
                    last_value = either::Left(value);
                }
                (ConfigPathElement::Index(i), either::Left(toml::Value::Array(arr))) => {
                    let Some(value) = arr.get(i as usize) else {
                        return Ok(false);
                    };
                    last_value = either::Left(value);
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    pub fn changed(&self) -> bool {
        let ConfigLoadState::Loaded { table: _, changed } = &**self.config.load() else {
            return false;
        };
        changed.load(Ordering::Acquire)
    }

    pub async fn save(&self) -> Result<(), ConfigSaveError> {
        let ConfigLoadState::Loaded { table, changed } = &**self.config.load() else {
            return Ok(());
        };
        changed.store(false, Ordering::Release);
        let table = table.lock().await;
        let config_string = toml::to_string_pretty(&*table).map_err(ConfigSaveError::Serialize)?;
        // Avoid holding the lock while saving to the file
        drop(table);
        tokio::fs::write(&self.path, config_string.as_bytes())
            .await
            .map_err(ConfigSaveError::Io)?;
        Ok(())
    }
}

impl From<toml::Value> for ConfigTree {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(v) => ConfigTree {
                nodes: vec![ConfigValue::String(v)],
                root_id: 0,
            },
            toml::Value::Integer(v) => ConfigTree {
                nodes: vec![ConfigValue::S64(v)],
                root_id: 0,
            },
            toml::Value::Float(v) => ConfigTree {
                nodes: vec![ConfigValue::F64(v)],
                root_id: 0,
            },
            toml::Value::Boolean(v) => ConfigTree {
                nodes: vec![ConfigValue::Bool(v)],
                root_id: 0,
            },
            toml::Value::Datetime(v) => ConfigTree {
                nodes: vec![ConfigValue::String(v.to_string())],
                root_id: 0,
            },
            toml::Value::Array(v) => {
                let mut node_indicies = Vec::new();
                let mut nodes =
                    v.into_iter()
                        .map(ConfigTree::from)
                        .fold(Vec::new(), |mut vec, mut tree| {
                            node_indicies.push(tree.root_id + vec.len() as u32);
                            vec.append(&mut tree.nodes);
                            vec
                        });
                nodes.push(ConfigValue::List(node_indicies));
                ConfigTree {
                    root_id: nodes.len() as u32 - 1,
                    nodes,
                }
            }
            toml::Value::Table(v) => {
                let mut map = Vec::new();
                let mut nodes = v.into_iter().map(|(k, v)| (k, ConfigTree::from(v))).fold(
                    Vec::new(),
                    |mut vec, (k, mut tree)| {
                        map.push((k, tree.root_id + vec.len() as u32));
                        vec.append(&mut tree.nodes);
                        vec
                    },
                );
                nodes.push(ConfigValue::Object(map));
                ConfigTree {
                    root_id: nodes.len() as u32 - 1,
                    nodes,
                }
            }
        }
    }
}

impl TryFrom<ConfigTree> for Option<toml::Value> {
    type Error = ();
    fn try_from(value: ConfigTree) -> Result<Self, Self::Error> {
        /// A node is `Some(...)` if it has not been seen before, otherwise `None`.
        fn try_from_inner(
            nodes: &mut Vec<Option<ConfigValue>>,
            current_node: ConfigValue,
        ) -> Result<Option<toml::Value>, ()> {
            Ok(match current_node {
                ConfigValue::String(v) => Some(toml::Value::String(v)),
                ConfigValue::S64(v) => Some(toml::Value::Integer(v)),
                ConfigValue::F64(v) => Some(toml::Value::Float(v)),
                ConfigValue::Bool(v) => Some(toml::Value::Boolean(v)),
                ConfigValue::Option(None) => None,
                ConfigValue::Option(Some(id)) => {
                    let Some(node) = nodes.get_mut(id as usize).and_then(Option::take) else {
                        return Err(());
                    };
                    try_from_inner(nodes, node)?
                }
                ConfigValue::List(ids) => {
                    let mut arr = Vec::with_capacity(ids.len());
                    for id in ids {
                        let Some(node) = nodes.get_mut(id as usize).and_then(Option::take) else {
                            return Err(());
                        };
                        let Some(value) = try_from_inner(nodes, node)? else {
                            // Omit null values from lists since TOML does not support them.
                            continue;
                        };
                        arr.push(value);
                    }
                    Some(toml::Value::Array(arr))
                }
                ConfigValue::Object(keys_ids) => {
                    let mut map = Map::new();
                    for (key, id) in keys_ids {
                        let Some(node) = nodes.get_mut(id as usize).and_then(Option::take) else {
                            return Err(());
                        };
                        let Some(value) = try_from_inner(nodes, node)? else {
                            // Omit null values from objects since TOML does not support them.
                            continue;
                        };
                        map.insert(key, value);
                    }
                    Some(toml::Value::Table(map))
                }
            })
        }

        let mut nodes = value.nodes.into_iter().map(Some).collect::<Vec<_>>();
        let Some(root_node) = nodes.get_mut(value.root_id as usize).and_then(Option::take) else {
            return Err(());
        };
        try_from_inner(&mut nodes, root_node)
    }
}

fn index_table(table: &Table, k: ConfigPath) -> Option<either::Either<&toml::Value, &Table>> {
    let mut current_value: Option<&toml::Value> = None;
    for elem in k {
        match elem {
            ConfigPathElement::Key(name) => {
                let table = if current_value.is_none() {
                    table
                } else if let Some(toml::Value::Table(table)) = current_value {
                    table
                } else {
                    return None;
                };
                current_value = Some(table.get(&name)?);
            }
            ConfigPathElement::Index(i) => {
                let Some(toml::Value::Array(arr)) = current_value else {
                    return None;
                };
                current_value = Some(arr.get(i as usize)?);
            }
        }
    }
    Some(if let Some(value) = current_value {
        either::Left(value)
    } else {
        either::Right(table)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::{Value, map::Map};

    #[test]
    fn value_to_tree() {
        {
            let value = Value::Table({
                let mut map = Map::new();
                map.insert(
                    "a".to_owned(),
                    Value::Array(vec![Value::String("hi".to_owned()), Value::Integer(1)]),
                );
                map.insert("b".to_owned(), Value::String("some value".to_owned()));
                map
            });
            let tree = ConfigTree::from(value.clone());
            let value_from_tree = <Option<Value>>::try_from(tree).unwrap().unwrap();
            assert_eq!(value, value_from_tree);
        }
    }

    #[test]
    fn value_from_tree() {
        {
            let tree = ConfigTree {
                nodes: vec![ConfigValue::Option(None)],
                root_id: 0,
            };
            let value_from_tree = <Option<Value>>::try_from(tree).unwrap();
            assert!(value_from_tree.is_none());
        }
        {
            let tree = ConfigTree {
                nodes: vec![
                    ConfigValue::List(vec![1, 2]),
                    ConfigValue::String("hi".to_owned()),
                    ConfigValue::S64(123456),
                ],
                root_id: 0,
            };
            let value_from_tree = <Option<Value>>::try_from(tree).unwrap();
            assert_eq!(
                value_from_tree,
                Some(Value::Array(vec![
                    Value::String("hi".to_owned()),
                    Value::Integer(123456)
                ]))
            );
        }
    }

    #[test]
    fn reject_cycles() {
        let tree = ConfigTree {
            nodes: vec![
                ConfigValue::List(vec![1, 0]),
                ConfigValue::String("hi".to_owned()),
            ],
            root_id: 0,
        };
        assert!(<Option<Value>>::try_from(tree).is_err());
    }
}
