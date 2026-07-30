use std::collections::HashMap;

pub use crate::wit::pumpkin::plugin::config::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigVal {
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    List(Vec<ConfigVal>),
    Object(HashMap<String, ConfigVal>),
    Option(Option<Box<ConfigVal>>),
}

impl From<ConfigVal> for ConfigTree {
    fn from(value: ConfigVal) -> Self {
        match value {
            ConfigVal::String(v) => ConfigTree {
                nodes: vec![ConfigValue::String(v)],
                root_id: 0,
            },
            ConfigVal::I64(v) => ConfigTree {
                nodes: vec![ConfigValue::S64(v)],
                root_id: 0,
            },
            ConfigVal::F64(v) => ConfigTree {
                nodes: vec![ConfigValue::F64(v)],
                root_id: 0,
            },
            ConfigVal::Bool(v) => ConfigTree {
                nodes: vec![ConfigValue::Bool(v)],
                root_id: 0,
            },
            ConfigVal::List(v) => {
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
            ConfigVal::Object(v) => {
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
            ConfigVal::Option(v) => {
                if let Some(v) = v {
                    let ConfigTree { mut nodes, root_id } = ConfigTree::from(*v);
                    nodes.push(ConfigValue::Option(Some(root_id)));
                    ConfigTree {
                        root_id: nodes.len() as u32 - 1,
                        nodes,
                    }
                } else {
                    ConfigTree {
                        nodes: vec![ConfigValue::Option(None)],
                        root_id: 0,
                    }
                }
            }
        }
    }
}

impl From<ConfigTree> for ConfigVal {
    /// Convert from a `ConfigTree` to a `ConfigVal`.
    /// # Panics
    /// Panics if the provided `ConfigTree` is invalid.
    fn from(value: ConfigTree) -> Self {
        /// A node is `Some(...)` if it has not been seen before, otherwise `None`.
        fn try_from_inner(
            nodes: &mut Vec<Option<ConfigValue>>,
            current_node: ConfigValue,
        ) -> Result<ConfigVal, ()> {
            Ok(match current_node {
                ConfigValue::String(v) => ConfigVal::String(v),
                ConfigValue::S64(v) => ConfigVal::I64(v),
                ConfigValue::F64(v) => ConfigVal::F64(v),
                ConfigValue::Bool(v) => ConfigVal::Bool(v),
                ConfigValue::Option(None) => ConfigVal::Option(None),
                ConfigValue::Option(Some(id)) => {
                    let Some(node) = nodes.get_mut(id as usize).and_then(Option::take) else {
                        return Err(());
                    };
                    ConfigVal::Option(Some(Box::new(try_from_inner(nodes, node)?)))
                }
                ConfigValue::List(ids) => {
                    let mut arr = Vec::with_capacity(ids.len());
                    for id in ids {
                        let Some(node) = nodes.get_mut(id as usize).and_then(Option::take) else {
                            return Err(());
                        };
                        arr.push(try_from_inner(nodes, node)?);
                    }
                    ConfigVal::List(arr)
                }
                ConfigValue::Object(keys_ids) => {
                    let mut map = HashMap::new();
                    for (key, id) in keys_ids {
                        let Some(node) = nodes.get_mut(id as usize).and_then(Option::take) else {
                            return Err(());
                        };
                        map.insert(key, try_from_inner(nodes, node)?);
                    }
                    ConfigVal::Object(map)
                }
            })
        }

        let mut nodes = value.nodes.into_iter().map(Some).collect::<Vec<_>>();
        let Some(root_node) = nodes.get_mut(value.root_id as usize).and_then(Option::take) else {
            panic!("Invalid config tree");
        };
        try_from_inner(&mut nodes, root_node).expect("Invalid config tree")
    }
}
