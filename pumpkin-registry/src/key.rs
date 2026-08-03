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
    #[allow(clippy::new_ret_no_self)]
    pub fn new(identifier: Identifier) -> DataKeyBuilder<T> {
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
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.keys[0]
    }

    #[must_use]
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
