use crate::bootstrap::BootstrapManager;
use pumpkin_util::identifier::Identifier;
use std::any::{Any, TypeId};
use std::sync::{Arc, OnceLock};

mod builder;
mod config;
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
pub use config::RegistryConfig;

pub static BOOTSTRAP: OnceLock<BootstrapManager> = OnceLock::new();

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

    /// Type-erased iteration over this registry.
    fn iter_erased(&self) -> ErasedRegistryIterator<'_>;

    /// Identifier lookup used while building a data key.
    fn get_id(&self, identifier: &Identifier) -> Option<usize>;
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

pub static ROOT: OnceLock<FrozenRegistry<Arc<dyn Registry>>> = OnceLock::new();
