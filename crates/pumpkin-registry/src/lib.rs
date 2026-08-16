use crate::bootstrap::BootstrapManager;
use pumpkin_util::identifier::Identifier;
use std::any::{Any, TypeId};
use std::pin::Pin;
use std::sync::{Arc, OnceLock};

mod builder;
mod immutable;
mod mutable;
mod resolvable;
mod r#static;

mod key;
mod value;

pub mod bootstrap;
pub mod error;
pub use crate::immutable::FrozenRegistry;
pub use crate::key::DataKey;
pub use crate::mutable::ReloadableRegistry;
pub use crate::resolvable::{RegistryResolvable, RegistryResolvableSet};
pub use crate::r#static::StaticRegistry;
pub use crate::value::{DataKeyRef, ErasedRegistryRef};
pub use builder::RegistryBuilder;

pub static BOOTSTRAP: OnceLock<BootstrapManager> = OnceLock::new();

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub type ErasedRegistryIterator<'a> =
    Box<dyn Iterator<Item = (Identifier, ErasedRegistryRef<'a>)> + 'a>;

pub trait Registry: Any + Send + Sync {
    fn arc_dyn(self) -> Arc<dyn Registry>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
    fn item_type_id(&self) -> TypeId;
    fn item_type_name(&self) -> &'static str;

    /// Type-erased lookup used when walking registry trees.
    fn by_id_erased(&self, id: usize) -> Option<value::ErasedRegistryRef<'_>>;

    /// Async type-erased lookup used when walking registry trees.
    fn by_id_erased_async(&self, id: usize) -> BoxFuture<'_, Option<ErasedRegistryRef<'_>>>;

    /// Type-erased iteration over this registry.
    fn iter_erased(&self) -> ErasedRegistryIterator<'_>;

    /// Blocking identifier lookup used while building a data key.
    fn get_id(&self, identifier: &Identifier) -> Option<usize>;

    /// Async identifier lookup used while building a data key.
    fn get_id_async<'a>(&'a self, identifier: &'a Identifier) -> BoxFuture<'a, Option<usize>>;
}

pub trait TypedRegistry<'a>: Registry {
    type Item;
    type Iter: Iterator<Item = Self::IterItem> + 'a;
    type IterItem;

    fn get(&'a self, identifier: &Identifier) -> Option<Self::Item> {
        self.get_id(identifier).and_then(|id| self.by_id(id))
    }

    fn by_id(&'a self, id: usize) -> Option<Self::Item>;

    fn iter(&'a self) -> Self::Iter;
}

pub trait AsyncTypedRegistry<'a>: TypedRegistry<'a> {
    fn get(&'a self, identifier: &'a Identifier) -> BoxFuture<'a, Option<Self::Item>> {
        Box::pin(async move {
            let id = self.get_id_async(identifier).await?;
            AsyncTypedRegistry::by_id(self, id).await
        })
    }

    fn by_id(&'a self, id: usize) -> BoxFuture<'a, Option<Self::Item>>;

    #[allow(clippy::iter_not_returning_iterator)]
    fn iter(&'a self) -> BoxFuture<'a, Self::Iter>;
}

pub static ROOT: OnceLock<FrozenRegistry<Arc<dyn Registry>>> = OnceLock::new();
