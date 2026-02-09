use crate::serialization::dynamic_ops::DynamicOps;
use std::collections::HashMap;

/// A struct to compress keys of a map of type `T` by converting them to numbers and back.
pub struct KeyCompressor<T, O: DynamicOps<Value = T> + 'static> {
    compress_map: HashMap<String, usize>,
    decompress_map: HashMap<usize, String>,
    size: usize,
    ops: &'static O,
}

impl<T, O: DynamicOps<Value = T>> KeyCompressor<T, O> {
    /// Returns a new `KeyCompressor` with the calculated compressor and decompressor maps.
    fn new(ops: &'static O, key_iter: impl Iterator<Item = T>) -> Self {
        let mut c = Self {
            compress_map: HashMap::new(),
            decompress_map: HashMap::new(),
            size: 0,
            ops,
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

    /// Gets the decompressed key of an index.
    pub fn decompress_key(&self, key: usize) -> Option<T> {
        self.decompress_map
            .get(&key)
            .map(|s| self.ops.create_string(s))
    }

    /// Gets the compressed key of the dynamic type.
    pub fn compress_key(&self, key: &T) -> Option<usize> {
        let string = self.ops.get_string(key).into_result()?;
        self.compress_key_str(&string)
    }

    /// Gets the compressed key of a string value.
    pub fn compress_key_str(&self, key: &str) -> Option<usize> {
        self.compress_map.get(key).copied()
    }

    /// Returns the size of the compressed/decompressed maps.
    pub const fn size(&self) -> usize {
        self.size
    }
}
