use crate::{
    BoxFuture, BoxedRegistry, ImmutableRegistry, Registry,
    builder::RegistryBuilder,
    error::{RegistryInitError, RegistryInsertError},
    value::{ErasedRegistryRef, LockedIterator, RegistryRef},
};
use pumpkin_util::identifier::Identifier;
use std::any::{Any, TypeId, type_name};
use tokio::sync::{RwLock, RwLockReadGuard};
pub struct MutableRegistry<T: Send + Sync + 'static>(pub(crate) RwLock<RegistryBuilder<T>>);

impl<T: Send + Sync + 'static> MutableRegistry<T> {
    pub fn new(
        static_entries: &'static [T],
        static_identifiers: &[Identifier],
    ) -> Result<Self, RegistryInitError> {
        Ok(Self(RwLock::new(RegistryBuilder::new(
            static_entries,
            static_identifiers,
        )?)))
    }

    pub async fn register(
        &self,
        identifier: Identifier,
        value: T,
    ) -> Result<(), RegistryInsertError> {
        self.0.write().await.register(identifier, value)
    }

    #[must_use]
    pub async fn get(&self, identifier: &Identifier) -> Option<RegistryRef<'_, T>> {
        RwLockReadGuard::try_map(self.0.read().await, |registry| registry.get(identifier))
            .map(RegistryRef::Locked)
            .ok()
    }

    #[must_use]
    pub async fn get_by_id(&self, id: usize) -> Option<RegistryRef<'_, T>> {
        RwLockReadGuard::try_map(self.0.read().await, |registry| registry.get_by_id(id))
            .map(RegistryRef::Locked)
            .ok()
    }

    #[must_use]
    pub async fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.0.read().await.get_id(identifier)
    }

    #[must_use]
    pub async fn contains(&self, identifier: &Identifier) -> bool {
        self.0.read().await.contains(identifier)
    }

    #[must_use]
    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    #[must_use]
    pub async fn is_empty(&self) -> bool {
        self.0.read().await.is_empty()
    }

    #[allow(clippy::iter_not_returning_iterator)] // does clippy know how async works?
    pub async fn iter(&self) -> impl Iterator<Item = (&Identifier, &T)> {
        LockedIterator::new(self.0.read().await)
    }
}

impl MutableRegistry<BoxedRegistry> {
    async fn into_nested_immutable(self) -> ImmutableRegistry<BoxedRegistry> {
        let RegistryBuilder {
            static_entries,
            entries,
            mapping,
        } = self.0.into_inner();

        let mut immutable_entries = Vec::with_capacity(entries.len());

        for entry in entries {
            immutable_entries.push(entry.into_immutable().await);
        }

        ImmutableRegistry::new(
            static_entries,
            immutable_entries.into_boxed_slice(),
            mapping,
        )
    }
}

impl<T: Send + Sync + 'static> Registry for MutableRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn get_id<'a>(&'a self, identifier: &'a Identifier) -> BoxFuture<'a, Option<usize>> {
        Box::pin(async move { Self::get_id(self, identifier).await })
    }

    fn get_by_id(&self, id: usize) -> BoxFuture<'_, Option<ErasedRegistryRef<'_>>> {
        Box::pin(async move { Self::get_by_id(self, id).await.map(ErasedRegistryRef::new) })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_immutable(self: Box<Self>) -> BoxFuture<'static, BoxedRegistry> {
        Box::pin(async move {
            let erased: Box<dyn Any + Send> = self;

            match erased.downcast::<MutableRegistry<BoxedRegistry>>() {
                Ok(registry) => Box::new(registry.into_nested_immutable().await) as BoxedRegistry,
                Err(erased) => {
                    let registry = erased
                        .downcast::<Self>()
                        .expect("downcast back to MutableRegistry<T> must succeed");

                    Box::new(ImmutableRegistry::from(*registry)) as BoxedRegistry
                }
            }
        })
    }
}
