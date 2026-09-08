use std::collections::HashMap;

use crate::wit::pumpkin::plugin::config::{ConfigTree, ConfigValue};

impl From<bool> for ConfigTree {
    fn from(value: bool) -> Self {
        Self {
            nodes: vec![ConfigValue::Bool(value)],
            root_id: 0,
        }
    }
}

impl From<String> for ConfigTree {
    fn from(value: String) -> Self {
        Self {
            nodes: vec![ConfigValue::String(value)],
            root_id: 0,
        }
    }
}

impl From<&str> for ConfigTree {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<i64> for ConfigTree {
    fn from(value: i64) -> Self {
        Self {
            nodes: vec![ConfigValue::S64(value)],
            root_id: 0,
        }
    }
}

impl From<f64> for ConfigTree {
    fn from(value: f64) -> Self {
        Self {
            nodes: vec![ConfigValue::F64(value)],
            root_id: 0,
        }
    }
}

impl<T> From<Option<T>> for ConfigTree
where
    Self: From<T>,
{
    fn from(value: Option<T>) -> Self {
        if let Some(value) = value {
            value.into()
        } else {
            Self {
                nodes: vec![ConfigValue::Null],
                root_id: 0,
            }
        }
    }
}

impl<T> From<Vec<T>> for ConfigTree
where
    Self: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let mut node_indices = Vec::new();
        let mut nodes = value
            .into_iter()
            .map(Self::from)
            .fold(Vec::new(), |mut vec, mut tree| {
                node_indices.push(tree.root_id + vec.len() as u32);
                vec.append(&mut tree.nodes);
                vec
            });
        nodes.push(ConfigValue::List(node_indices));
        Self {
            root_id: nodes.len() as u32 - 1,
            nodes,
        }
    }
}

impl<T> From<HashMap<String, T>> for ConfigTree
where
    Self: From<T>,
{
    fn from(value: HashMap<String, T>) -> Self {
        let mut map = Vec::new();
        let mut nodes = value.into_iter().map(|(k, v)| (k, Self::from(v))).fold(
            Vec::new(),
            |mut vec, (k, mut tree)| {
                map.push((k, tree.root_id + vec.len() as u32));
                vec.append(&mut tree.nodes);
                vec
            },
        );
        nodes.push(ConfigValue::Object(map));
        Self {
            root_id: nodes.len() as u32 - 1,
            nodes,
        }
    }
}
