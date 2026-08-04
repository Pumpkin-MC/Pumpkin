use std::{
    any::{TypeId, type_name},
    fmt::Display,
    marker::PhantomData,
    sync::Arc,
};

use pumpkin_util::identifier::Identifier;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DataKey<T: ?Sized + Send + Sync + 'static> {
    keys: Arc<[Identifier]>,
    marker: PhantomData<T>,
}

impl<T: ?Sized + Send + Sync + 'static> DataKey<T> {
    #[must_use]
    pub fn builder(identifier: Identifier) -> DataKeyBuilder<T> {
        DataKeyBuilder {
            keys: vec![identifier],
            marker: PhantomData,
        }
    }

    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.keys[0]
    }

    #[must_use]
    pub fn path(&self) -> &[Identifier] {
        &self.keys
    }

    #[must_use]
    pub fn child<U: Send + Sync + 'static>(&self, identifier: Identifier) -> DataKey<U> {
        let mut keys = self.keys.to_vec();
        keys.push(identifier);

        DataKey {
            keys: keys.into(),
            marker: PhantomData,
        }
    }

    #[must_use]
    pub fn erased(&self) -> ErasedDataKey {
        ErasedDataKey {
            keys: self.keys.clone(),
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }
}

impl<T: ?Sized + Send + Sync + 'static> Clone for DataKey<T> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized + Send + Sync + 'static> Display for DataKey<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = self.keys.iter();

        if let Some(first) = keys.next() {
            write!(formatter, "{first}")?;

            for identifier in keys {
                write!(formatter, "/{identifier}")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErasedDataKey {
    keys: Arc<[Identifier]>,
    type_id: std::any::TypeId,
    type_name: &'static str,
}

impl ErasedDataKey {
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn identifier(&self) -> &Identifier {
        &self.keys[0]
    }

    pub fn path(&self) -> &[Identifier] {
        &self.keys
    }
}

impl Display for ErasedDataKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = self.keys.iter();

        if let Some(first) = keys.next() {
            write!(formatter, "{first}")?;

            for identifier in keys {
                write!(formatter, "/{identifier}")?;
            }
        }

        Ok(())
    }
}
pub struct DataKeyBuilder<T: ?Sized + Send + Sync + 'static> {
    keys: Vec<Identifier>,
    marker: PhantomData<T>,
}

impl<T: ?Sized + Send + Sync + 'static> DataKeyBuilder<T> {
    pub fn add_subkey(mut self, identifier: Identifier) -> Self {
        self.keys.push(identifier);
        self
    }

    pub fn build(self) -> DataKey<T> {
        DataKey {
            keys: self.keys.into(),
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> Identifier {
        Identifier::from_static("test", value)
    }

    #[test]
    fn builder_creates_single_element_path() {
        let key = DataKey::<u32>::builder(id("blocks")).build();

        assert_eq!(key.identifier(), &id("blocks"));
        assert_eq!(key.path(), &[id("blocks")]);
        assert_eq!(key.to_string(), "test:blocks");
    }

    #[test]
    fn builder_preserves_subkey_order() {
        let key = DataKey::<String>::builder(id("root"))
            .add_subkey(id("registries"))
            .add_subkey(id("items"))
            .build();

        assert_eq!(key.path(), &[id("root"), id("registries"), id("items")]);
        assert_eq!(key.to_string(), "test:root/test:registries/test:items");
    }

    #[test]
    fn child_appends_identifier_and_changes_value_type() {
        let parent = DataKey::<u32>::builder(id("numbers")).build();
        let child = parent.child::<String>(id("name"));

        assert_eq!(parent.path(), &[id("numbers")]);
        assert_eq!(child.path(), &[id("numbers"), id("name")]);
        assert_eq!(child.erased().type_id(), TypeId::of::<String>());
    }

    #[test]
    fn clone_shares_the_same_logical_path() {
        let key = DataKey::<[u8]>::builder(id("root"))
            .add_subkey(id("bytes"))
            .build();
        let cloned = key.clone();

        assert_eq!(key, cloned);
        assert_eq!(key.path(), cloned.path());
    }

    #[test]
    fn erased_key_retains_path_and_type_information() {
        let key = DataKey::<str>::builder(id("strings"))
            .add_subkey(id("hello"))
            .build();
        let erased = key.erased();

        assert_eq!(erased.identifier(), &id("strings"));
        assert_eq!(erased.path(), key.path());
        assert_eq!(erased.type_id(), TypeId::of::<str>());
        assert_eq!(erased.type_name(), type_name::<str>());
        assert_eq!(erased.to_string(), key.to_string());
    }

    #[test]
    fn different_generic_types_produce_different_erased_keys() {
        let number = DataKey::<u32>::builder(id("value")).build().erased();
        let text = DataKey::<String>::builder(id("value")).build().erased();

        assert_ne!(number, text);
    }
}
