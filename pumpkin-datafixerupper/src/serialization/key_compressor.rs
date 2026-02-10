use crate::serialization::dynamic_ops::DynamicOps;
use std::collections::HashMap;

/// A struct to compress keys of a map by converting them to numbers (making a kind of list) and back.
pub struct KeyCompressor {
    compress_map: HashMap<String, usize>,
    decompress_map: HashMap<usize, String>,
    size: usize,
}

impl KeyCompressor {
    /// Returns a new `KeyCompressor` with the calculated compressor and decompressor maps .
    pub(crate) fn new<T>(
        key_iter: impl Iterator<Item = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> Self {
        let mut c = Self {
            compress_map: HashMap::new(),
            decompress_map: HashMap::new(),
            size: 0,
        };

        // Iterate over every key.
        key_iter
            .filter_map(|dynamic_key| ops.get_string(&dynamic_key).into_result())
            .for_each(|key| {
                if c.compress_map.contains_key(&key) {
                    return;
                }
                // The index that the key will correspond to.
                let i = c.size;
                c.compress_map.insert(key.clone(), i);
                c.decompress_map.insert(i, key);
            });

        c
    }

    /// Gets the decompressed key of an index with the provided dynamic type.
    pub fn decompress_key<T>(
        &self,
        key: usize,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> Option<T> {
        self.decompress_map.get(&key).map(|s| ops.create_string(s))
    }

    /// Gets the compressed key of the provided dynamic type.
    pub fn compress_key<T>(
        &self,
        key: &T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> Option<usize> {
        let string = ops.get_string(key).into_result()?;
        self.compress_key_str(&string)
    }

    /// Gets the decompressed string key of an index.
    fn decompress_key_str(&self, key: usize) -> Option<String> {
        self.decompress_map.get(&key).cloned()
    }

    /// Gets the compressed key of a string value.
    pub(crate) fn compress_key_str(&self, key: &str) -> Option<usize> {
        self.compress_map.get(key).copied()
    }

    /// Returns the size of the compressed/decompressed maps.
    pub const fn size(&self) -> usize {
        self.size
    }
}
