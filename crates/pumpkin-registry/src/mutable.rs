use crate::{
    AsyncTypedRegistry, BOOTSTRAP, BoxFuture, Registry, TypedRegistry,
    error::BootstrapError,
    immutable::FrozenRegistry,
    value::{DynIterator, ErasedRegistryRef, LockedIterator},
};
use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;
use std::any::{Any, TypeId, type_name};
use tokio::sync::{RwLock, RwLockReadGuard};

pub struct ReloadableRegistry<T: Send + Sync + 'static> {
    inner: RwLock<FrozenRegistry<T>>,
    name: Identifier,
}

impl<T: Send + Sync + 'static> ReloadableRegistry<T> {
    pub(crate) fn new(
        name: Identifier,
        entries: Box<[T]>,
        mapping: FxHashMap<Identifier, usize>,
    ) -> Self {
        Self {
            inner: RwLock::new(FrozenRegistry::new(entries, mapping)),
            name,
        }
    }

    pub async fn reload(&self) -> Result<(), BootstrapError> {
        let (entries, mapping) = BOOTSTRAP.populate::<T>(&self.name)?;
        let new_inner = FrozenRegistry::new(entries.into_boxed_slice(), mapping);
        *self.inner.write().await = new_inner;
        Ok(())
    }

    pub fn blocking_reload(&self) -> Result<(), BootstrapError> {
        let (entries, mapping) = BOOTSTRAP.populate::<T>(&self.name)?;
        let new_inner = FrozenRegistry::new(entries.into_boxed_slice(), mapping);
        *self.inner.blocking_write() = new_inner;
        Ok(())
    }

    // some way to swap out the FrozenRegistry for datapack reloads
}

impl<T: Send + Sync + 'static> Registry for ReloadableRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.inner.blocking_read().get_id(identifier)
    }

    fn get_id_async<'a>(&'a self, identifier: &'a Identifier) -> BoxFuture<'a, Option<usize>> {
        Box::pin(async move { self.inner.read().await.get_id(identifier) })
    }

    fn by_id_erased(&self, id: usize) -> Option<ErasedRegistryRef<'_>> {
        RwLockReadGuard::try_map(self.inner.blocking_read(), |a| {
            a.by_id(id).map(|v| v as &dyn Any)
        })
        .ok()
        .map(ErasedRegistryRef::Locked)
    }

    fn by_id_erased_async(&self, id: usize) -> BoxFuture<'_, Option<ErasedRegistryRef<'_>>> {
        Box::pin(async move {
            RwLockReadGuard::try_map(self.inner.read().await, |a| {
                a.by_id(id).map(|v| v as &dyn Any)
            })
            .ok()
            .map(ErasedRegistryRef::Locked)
        })
    }
}

impl<'a, T: Send + Sync + 'static> TypedRegistry<'a> for ReloadableRegistry<T> {
    type Item = RwLockReadGuard<'a, T>;
    type IterItem = (&'a Identifier, &'a T);
    type Iter = DynIterator<'a, Self::IterItem>;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        RwLockReadGuard::try_map(self.inner.blocking_read(), |a| a.by_id(id)).ok()
    }

    fn iter(&'a self) -> Self::Iter {
        DynIterator::new(LockedIterator::new(self.inner.blocking_read()))
    }
}

impl<'a, T: Send + Sync + 'static> AsyncTypedRegistry<'a> for ReloadableRegistry<T> {
    fn by_id(&'a self, id: usize) -> BoxFuture<'a, Option<Self::Item>> {
        Box::pin(
            async move { RwLockReadGuard::try_map(self.inner.read().await, |a| a.by_id(id)).ok() },
        )
    }

    fn iter(&'a self) -> BoxFuture<'a, Self::Iter> {
        Box::pin(async move { DynIterator::new(LockedIterator::new(self.inner.read().await)) })
    }
}
