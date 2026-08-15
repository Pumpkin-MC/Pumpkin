use std::collections::HashMap;

pub use crate::wit::pumpkin::plugin::config::*;

/// A value in the configuration.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigVal {
    /// A boolean value.
    Bool(bool),
    /// An integer value.
    I64(i64),
    /// A float value.
    F64(f64),
    /// A string value.
    String(String),
    /// A list of values which may have different types. If the underlying format does not support
    /// different-type lists, trying to create/set such a list will result in an error.
    List(Vec<ConfigVal>),
    /// An object/map, a key-value mapping.
    Object(HashMap<String, ConfigVal>),
    /// If the underlying format does not support `null` values (such as TOML), the value
    /// is omitted. If the format supports `null` (such as JSON), the `null` is set as the value.
    Null,
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
                let mut node_indices = Vec::new();
                let mut nodes =
                    v.into_iter()
                        .map(ConfigTree::from)
                        .fold(Vec::new(), |mut vec, mut tree| {
                            node_indices.push(tree.root_id + vec.len() as u32);
                            vec.append(&mut tree.nodes);
                            vec
                        });
                nodes.push(ConfigValue::List(node_indices));
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
            ConfigVal::Null => ConfigTree {
                nodes: vec![ConfigValue::Null],
                root_id: 0,
            },
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
                ConfigValue::Null => ConfigVal::Null,
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

impl PartialEq<bool> for ConfigVal {
    fn eq(&self, other: &bool) -> bool {
        let Self::Bool(value) = self else {
            return false;
        };
        value == other
    }
}

impl PartialEq<&str> for ConfigVal {
    fn eq(&self, other: &&str) -> bool {
        let Self::String(s) = self else {
            return false;
        };

        s == other
    }
}

impl PartialEq<String> for ConfigVal {
    fn eq(&self, other: &String) -> bool {
        let Self::String(s) = self else {
            return false;
        };

        s == other
    }
}

impl PartialEq<i64> for ConfigVal {
    fn eq(&self, other: &i64) -> bool {
        let Self::I64(i) = self else {
            return false;
        };
        i == other
    }
}

impl PartialEq<f64> for ConfigVal {
    fn eq(&self, other: &f64) -> bool {
        let Self::F64(f) = self else {
            return false;
        };
        f == other
    }
}

impl<T> PartialEq<Vec<T>> for ConfigVal
where
    Self: PartialEq<T>,
{
    fn eq(&self, other: &Vec<T>) -> bool {
        let Self::List(vec) = self else {
            return false;
        };
        if other.len() != vec.len() {
            return false;
        }
        for (a, b) in vec.iter().zip(other) {
            if a != b {
                return false;
            }
        }
        true
    }
}

impl<T> PartialEq<HashMap<String, T>> for ConfigVal
where
    Self: PartialEq<T>,
{
    fn eq(&self, other: &HashMap<String, T>) -> bool {
        let Self::Object(o) = self else {
            return false;
        };
        if o.len() != other.len() {
            return false;
        }
        for (k, v) in o {
            let Some(b) = other.get(k) else {
                return false;
            };
            if v != b {
                return false;
            }
        }
        true
    }
}

impl<T> PartialEq<Option<T>> for ConfigVal
where
    Self: PartialEq<T>,
{
    fn eq(&self, other: &Option<T>) -> bool {
        if let Some(other) = other {
            self == other
        } else {
            matches!(self, ConfigVal::Null)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equalities() {
        assert_eq!(ConfigVal::Null, None::<bool>);
        assert_eq!(ConfigVal::List(Vec::new()), Some(Vec::<String>::new()));
        assert_ne!(
            ConfigVal::List(vec![ConfigVal::Bool(true), ConfigVal::Bool(false)]),
            vec![true, false, false]
        );
        assert_ne!(
            ConfigVal::List(vec![ConfigVal::Bool(true), ConfigVal::Bool(false)]),
            vec![true]
        );
        assert_eq!(ConfigVal::String("abcdefg".to_owned()), "abcdefg");
        assert_ne!(ConfigVal::String("abcdefg".to_owned()), None::<String>);
    }
}
